use anyhow::Result;
use rand::Rng;
use rand::rngs::StdRng;
use sonettobuf::{ActEffect, BeginRoundOper, Fight, FightStep, fight_step};
use std::sync::Arc;

use crate::state::battle::{
    effects::effect_types::EffectType, manager::skill_mgr::FightSkillMgr, round::RoundState,
};

#[derive(Default, Debug, Clone)]
pub struct FightCardMgr {
    fight: Arc<Fight>,
    skill_mgr: FightSkillMgr,
}

impl FightCardMgr {
    pub fn new(fight: Arc<Fight>) -> Self {
        Self {
            skill_mgr: FightSkillMgr::new(fight.clone()),
            fight,
        }
    }

    pub fn update_fight(&mut self, fight: Arc<Fight>) {
        self.skill_mgr.update_fight(fight.clone());
        self.fight = fight;
    }

    pub fn create_refresh_step(&self, state: &RoundState) -> FightStep {
        FightStep {
            act_type: Some(fight_step::ActType::Effect.into()),
            from_id: Some(0),
            to_id: Some(0),
            act_id: Some(0),
            act_effect: vec![
                ActEffect {
                    effect_type: Some(EffectType::CardsPush as i32), // 154
                    card_info_list: state.player_deck.clone(),
                    team_type: Some(0),
                    ..Default::default()
                },
                ActEffect {
                    effect_type: Some(EffectType::CardDeckNum as i32), // 310
                    effect_num: Some(state.power),
                    team_type: Some(1),
                    ..Default::default()
                },
            ],
            card_index: Some(0),
            support_hero_id: Some(0),
            fake_timeline: Some(false),
        }
    }

    pub async fn execute_operation(
        &self,
        rng: &mut StdRng,
        state: &mut RoundState,
        oper: BeginRoundOper,
    ) -> Result<FightStep> {
        match oper.oper_type.unwrap_or(0) {
            1 => self.play_card(rng, state, oper).await,
            2 => {
                if oper.to_id.unwrap_or(0) != 0 {
                    self.play_card(rng, state, oper).await
                } else {
                    Ok(self.select_card(oper))
                }
            }
            3 => Ok(self.change_hero()),
            4 => Ok(self.end_turn()),
            _ => Ok(FightStep::default()),
        }
    }

    fn select_card(&self, oper: BeginRoundOper) -> FightStep {
        let card_index = oper.param1.unwrap_or(0);

        FightStep {
            act_type: Some(fight_step::ActType::Effect.into()),
            act_effect: vec![ActEffect {
                effect_type: Some(EffectType::AddHandCard as i32), // 149
                effect_num: Some(card_index),
                team_type: Some(1),
                ..Default::default()
            }],
            card_index: Some(card_index),
            ..Default::default()
        }
    }

    async fn play_card(
        &self,
        _rng: &mut StdRng,
        state: &mut RoundState,
        oper: BeginRoundOper,
    ) -> Result<FightStep> {
        let card_index = oper.param1.unwrap_or(0) as usize;
        let target_uid = oper.to_id.unwrap_or(0);

        let card = match state.player_deck.get(card_index).cloned() {
            Some(c) => c,
            None => return Ok(FightStep::default()),
        };

        // remove it from hand
        state.player_deck.remove(card_index);

        let hero_id = card.hero_id.unwrap_or(0);

        let caster_uid = state
            .iter_entities()
            .find(|e| e.model_id == Some(hero_id) && e.uid.unwrap_or(0) > 0)
            .and_then(|e| e.uid)
            .ok_or_else(|| anyhow::anyhow!("No entity for hero {}", hero_id))?;

        let skill_id = card.skill_id.unwrap_or(0);

        let mut step = self
            .skill_mgr
            .execute_skill(state, caster_uid, target_uid, skill_id)?;

        state.used_cards.push(card_index as i32);
        state.act_point = (state.act_point - 1).max(0);

        step.card_index = Some(card_index as i32);
        Ok(step)
    }

    pub async fn execute_ai_turn(
        &self,
        rng: &mut StdRng,
        state: &mut RoundState,
    ) -> Result<Vec<FightStep>> {
        let mut steps = Vec::new();

        let players: Vec<i64> = state
            .iter_entities()
            .filter(|e| e.uid.unwrap_or(0) > 0 && e.current_hp.unwrap_or(0) > 0)
            .filter_map(|e| e.uid)
            .collect();

        if players.is_empty() {
            return Ok(steps);
        }

        for i in 0..state.ai_cards.len() {
            let (caster_uid, skill_id, target_uid_opt) = {
                let card = &state.ai_cards[i];
                (
                    card.uid.unwrap_or(0),
                    card.skill_id.unwrap_or(0),
                    card.target_uid,
                )
            };

            if caster_uid >= 0 {
                continue;
            }

            let Some(caster) = state.get_entity(caster_uid) else {
                continue;
            };
            if caster.current_hp.unwrap_or(0) <= 0 {
                continue;
            }

            if skill_id == 0 {
                continue;
            }

            let target_uid = match target_uid_opt {
                Some(t) if t != 0 => t,
                _ => {
                    let t = players[rng.gen_range(0..players.len())];
                    state.ai_cards[i].target_uid = Some(t);
                    t
                }
            };

            let step = self
                .skill_mgr
                .execute_skill(state, caster_uid, target_uid, skill_id)?;

            steps.push(step);
        }

        Ok(steps)
    }

    fn change_hero(&self) -> FightStep {
        FightStep {
            act_type: Some(fight_step::ActType::Changehero.into()),
            ..Default::default()
        }
    }

    fn end_turn(&self) -> FightStep {
        FightStep {
            act_type: Some(fight_step::ActType::Effect.into()),
            act_effect: vec![ActEffect {
                effect_type: Some(EffectType::RoundEnd as i32), // 61
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}
