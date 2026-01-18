use std::sync::Arc;

use crate::state::battle::{
    effects::effect_types::EffectType, manager::{
        blood_pool_mgr::FightBloodPoolDataMgr, buff_mgr::BuffMgr,
        calculate_mgr::FightCalculateDataMgr, card_mgr::FightCardMgr,
        entity_mgr::FightEntityDataMgr, round_mgr::FightRoundMgr,
    }, mechanics::{
        Mechanics,
        bloodtithe::{BloodtitheState, fight_enables_bloodtithe},
    }, passives, step_builder::FightStepBuilder
};
use anyhow::Result;
use sonettobuf::{ActEffect, CardInfo, Fight, FightRound, FightStep};

#[derive(Default, Debug, Clone)]
pub struct FightDataMgr {
    fight: Arc<Fight>,
    mechanics: Mechanics,
    pub entity_mgr: FightEntityDataMgr,
    blood_pool_mgr: FightBloodPoolDataMgr,
    pub calculate_mgr: FightCalculateDataMgr,
    pub card_mgr: FightCardMgr,
    pub round_mgr: FightRoundMgr,
    pub buff_mgr: BuffMgr,
}

impl FightDataMgr {
    pub fn new(fight: Fight) -> Self {
        let fight_arc = Arc::new(fight);

        let mut mechanics = Mechanics::new();

        let mut blood_pool_mgr = FightBloodPoolDataMgr::new(fight_arc.clone());
        blood_pool_mgr.initialize(&mut mechanics.bloodtithe);

        Self {
            fight: fight_arc.clone(),
            mechanics,
            entity_mgr: FightEntityDataMgr::new(fight_arc.clone()),
            blood_pool_mgr,
            card_mgr: FightCardMgr::new(fight_arc.clone()),
            round_mgr: FightRoundMgr::new(fight_arc.clone()),
            calculate_mgr: FightCalculateDataMgr::new(fight_arc),
            buff_mgr: BuffMgr::new(),
        }
    }

    pub fn build_initial_round(
            &mut self,
            player_deck: Vec<CardInfo>,
            ai_deck: Vec<CardInfo>,
        ) -> Result<FightRound> {
            let mut batch: Vec<ActEffect> = vec![];
            let mut steps: Vec<FightStep> = vec![];
    
            let round_result = {
                let fight = Arc::make_mut(&mut self.fight);
    
                // Bootstrap effects (activity buffs with protected wrapping)
                let bootstrap_effects: Vec<ActEffect> = if let Some(attacker) = &fight.attacker {
                    let mut all = Vec::new();
                    for entity in &attacker.entitys {
                        all.extend(passives::build_bootstrap(entity)?);
                    }
                    all
                } else {
                    Vec::new()
                };
    
                if !bootstrap_effects.is_empty() {
                    let effs = process_effects(
                        bootstrap_effects,
                        fight,
                        &mut self.calculate_mgr,
                        &mut self.mechanics.bloodtithe,
                        &mut self.buff_mgr,
                    )?;
    
                    steps.push(FightStepBuilder::new_effect().add_effects(effs).build());
                }
    
                // Battle start effects (normal battle containers)
                let battle_start_effects: Vec<ActEffect> = if let Some(attacker) = &fight.attacker {
                    let mut all = Vec::new();
                    for entity in &attacker.entitys {
                        all.extend(passives::build_battle_start_passives(
                            entity,
                            fight,
                            &mut self.mechanics.bloodtithe,
                        )?);
                    }
                    all
                } else {
                    Vec::new()
                };
    
                if !battle_start_effects.is_empty() {
                    let effs = process_effects(
                        battle_start_effects,
                        fight,
                        &mut self.calculate_mgr,
                        &mut self.mechanics.bloodtithe,
                        &mut self.buff_mgr,
                    )?;
    
                    steps.push(FightStepBuilder::new_effect().add_effects(effs).build());
                }
    
                // Bloodtithe UI sync
                if fight_enables_bloodtithe(fight) {
                    let team = 1;
                    let bloodtithe_value = self.mechanics.bloodtithe.get_value(team);
                    let bloodtithe_max = self.mechanics.bloodtithe.get_max(team);
    
                    let display_uid = fight
                        .attacker
                        .as_ref()
                        .and_then(|a| a.entitys.iter().find(|e| e.team_type == Some(team)))
                        .and_then(|e| e.uid)
                        .unwrap_or(0);
    
                    let ui_step = FightStepBuilder::new_effect()
                        .add_bloodtithe_ui_sync(team, display_uid, bloodtithe_value, bloodtithe_max)
                        .build();
    
                    batch.extend(ui_step.act_effect);
                }
    
                // Round start effects
                let round_start_effects: Vec<ActEffect> = if let Some(attacker) = &fight.attacker {
                    let mut all = Vec::new();
                    for entity in &attacker.entitys {
                        all.extend(passives::build_round_start_passives(
                            entity,
                            fight,
                            &mut self.mechanics.bloodtithe,
                        )?);
                    }
                    all
                } else {
                    Vec::new()
                };
    
                if !round_start_effects.is_empty() {
                    batch.extend(round_start_effects);
                }
    
                // Enter fight deal
                batch.push(ActEffect {
                    effect_type: Some(EffectType::EnterFightDeal as i32), // 233
                    target_id: Some(0),
                    ..Default::default()
                });
    
                // Card deck num updates
                batch.push(ActEffect {
                    effect_type: Some(EffectType::CardDeckNum as i32), // 310
                    target_id: Some(0),
                    team_type: Some(1),
                    effect_num: Some(48),
                    ..Default::default()
                });
    
                batch.push(ActEffect {
                    effect_type: Some(EffectType::CardDeckNum as i32), // 310
                    target_id: Some(0),
                    team_type: Some(1),
                    effect_num: Some(48),
                    ..Default::default()
                });
    
                // Post power effects
                let post_power_effects: Vec<ActEffect> = if let Some(attacker) = &fight.attacker {
                    let mut all = Vec::new();
                    for entity in &attacker.entitys {
                        all.extend(passives::build_post_power_passives(entity, fight)?);
                    }
                    all
                } else {
                    Vec::new()
                };
    
                if !post_power_effects.is_empty() {
                    batch.extend(post_power_effects);
                }
    
                batch.push(ActEffect {
                    effect_type: Some(EffectType::CardDeckNum as i32), // 310
                    target_id: Some(0),
                    team_type: Some(1),
                    effect_num: Some(48),
                    ..Default::default()
                });
    
                steps.push(FightStepBuilder::new_effect().add_effects(batch).build());
    
                FightRound {
                    fight_step: steps,
                    act_point: Some(3),
                    is_finish: Some(false),
                    move_num: Some(0),
                    ex_point_info: self.calculate_mgr.build_ex_point_info(fight),
                    ai_use_cards: ai_deck,
                    power: Some(20),
                    skill_infos: self.calculate_mgr.build_player_skills(),
                    before_cards1: vec![],
                    team_a_cards1: player_deck,
                    before_cards2: vec![],
                    team_a_cards2: vec![],
                    next_round_begin_step: vec![],
                    use_card_list: vec![],
                    cur_round: Some(1),
                    hero_sp_attributes: self.calculate_mgr.build_hero_sp_attributes(fight),
                    last_change_hero_uid: Some(0),
                }
            };
    
            self.update_managers();
            Ok(round_result)
        }

    pub fn update_managers(&mut self) {
        let fight_arc = self.get_fight();

        self.entity_mgr.update_fight(fight_arc.clone());
        self.calculate_mgr.update_fight(fight_arc.clone());
        self.blood_pool_mgr.update_fight(fight_arc.clone());
        self.card_mgr.update_fight(fight_arc.clone());
        self.round_mgr.update_fight(fight_arc.clone());
    }

    pub fn get_fight(&self) -> Arc<Fight> {
        self.fight.clone()
    }

    pub fn get_fight_owned(&self) -> Fight {
        (*self.fight).clone()
    }

    pub fn get_fight_snapshot(&self) -> Arc<Fight> {
        self.get_fight()
    }
}

#[allow(dead_code)]
impl FightDataMgr {
    pub fn fight(&self) -> &Fight {
        &self.fight
    }

    pub fn fight_mut(&mut self) -> &mut Fight {
        Arc::make_mut(&mut self.fight)
    }

    pub fn bloodtithe(&self) -> &BloodtitheState {
        &self.mechanics.bloodtithe
    }

    pub fn bloodtithe_mut(&mut self) -> &mut BloodtitheState {
        &mut self.mechanics.bloodtithe
    }
}

fn process_effects(
    effects: Vec<ActEffect>,
    fight: &mut Fight,
    calculate_mgr: &mut FightCalculateDataMgr,
    bloodtithe: &mut BloodtitheState,
    buff_mgr: &mut BuffMgr,
) -> Result<Vec<ActEffect>> {
    for effect in &effects {
        calculate_mgr
            .play_act_effect_data(effect, fight, bloodtithe, buff_mgr)
            .map_err(|e| anyhow::anyhow!(e))?;
    }

    Ok(effects)
}

impl FightDataMgr {
    pub fn split_all_mut(
        &mut self,
    ) -> (
        &FightRoundMgr,
        &FightCardMgr,
        &mut FightCalculateDataMgr,
        &mut Fight,
        &mut BloodtitheState,
        &mut BuffMgr,
    ) {
        let fight = Arc::make_mut(&mut self.fight);

        (
            &self.round_mgr,
            &self.card_mgr,
            &mut self.calculate_mgr,
            fight,
            &mut self.mechanics.bloodtithe,
            &mut self.buff_mgr,
        )
    }
}
