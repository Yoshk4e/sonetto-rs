use anyhow::Result;
use rand::rngs::StdRng;
use sonettobuf::{
    ActEffect, BeginRoundOper, CardInfo, Fight, FightRound, FightStep,
    effect_type_enum::EffectType, fight_step,
};
use std::sync::Arc;

use crate::state::battle::{
    manager::{buff_mgr::BuffMgr, calculate_mgr::FightCalculateDataMgr, card_mgr::FightCardMgr},
    mechanics::bloodtithe::BloodtitheState,
    round::{RoundSnapshot, RoundState},
};

#[derive(Default, Debug, Clone)]
pub struct FightRoundMgr {
    fight: Arc<Fight>,
}

impl FightRoundMgr {
    pub fn new(fight: Arc<Fight>) -> Self {
        Self { fight }
    }

    pub fn update_fight(&mut self, fight: Arc<Fight>) {
        self.fight = fight;
    }

    pub async fn process_round(
        &self,
        rng: &mut StdRng,
        card_mgr: &FightCardMgr,
        calc: &mut FightCalculateDataMgr,
        fight: &mut Fight,
        bloodtithe: &mut BloodtitheState,
        operations: Vec<BeginRoundOper>,
        current_deck: Vec<CardInfo>,
        ai_deck: Vec<CardInfo>,
        buff_mgr: &mut BuffMgr,
    ) -> Result<FightRound> {
        let (steps, round_snapshot) = {
            let mut state = RoundState::new(&*fight)?;

            state.player_deck = current_deck.clone();
            state.ai_cards = ai_deck.clone();

            let mut steps = Vec::new();

            steps.push(card_mgr.create_refresh_step(&state));

            for oper in operations {
                let step = card_mgr.execute_operation(rng, &mut state, oper).await?;

                if step.act_type.unwrap_or(0) != 0 {
                    steps.push(step);
                }
            }

            if !state.is_finish {
                let ai_steps = card_mgr.execute_ai_turn(rng, &mut state).await?;
                steps.extend(ai_steps);
            }

            state.is_finish = self.check_battle_end(&state);

            (steps, state.export_snapshot())
        };

        calc.play_step_data_list(&steps, fight, bloodtithe, buff_mgr)
            .map_err(anyhow::Error::msg)?;
        calc.on_round_end();

        Ok(self.build_round_response(steps, round_snapshot, current_deck))
    }

    fn check_battle_end(&self, state: &RoundState) -> bool {
        let enemies_alive = state
            .iter_entities()
            .any(|e| e.uid.unwrap_or(0) < 0 && e.current_hp.unwrap_or(0) > 0);

        let heroes_alive = state
            .iter_entities()
            .any(|e| e.uid.unwrap_or(0) > 0 && e.current_hp.unwrap_or(0) > 0);

        !enemies_alive || !heroes_alive
    }

    pub fn build_round_response(
        &self,
        steps: Vec<FightStep>,
        snap: RoundSnapshot,
        before_cards: Vec<CardInfo>,
    ) -> FightRound {
        let wrapped_effects: Vec<ActEffect> = steps
            .into_iter()
            .map(|step| ActEffect {
                effect_type: Some(EffectType::Fightstep as i32),
                target_id: Some(0),
                effect_num: Some(0),
                fight_step: Some(step),
                ..Default::default()
            })
            .collect();

        let batched_step = FightStep {
            act_type: Some(fight_step::ActType::Effect.into()),
            from_id: Some(0),
            to_id: Some(0),
            act_id: Some(0),
            act_effect: wrapped_effects,
            card_index: Some(0),
            support_hero_id: Some(0),
            fake_timeline: Some(false),
        };

        FightRound {
            fight_step: vec![batched_step],
            act_point: Some(snap.act_point),
            is_finish: Some(snap.is_finish),
            move_num: Some(snap.move_num),
            ex_point_info: snap.ex_point_info,
            ai_use_cards: snap.ai_cards.clone(),
            power: Some(snap.power),
            skill_infos: vec![],
            before_cards1: before_cards,
            team_a_cards1: snap.player_deck.clone(),
            before_cards2: vec![],
            team_a_cards2: vec![],
            next_round_begin_step: vec![],
            use_card_list: snap.used_cards.clone(),
            cur_round: Some(snap.round_num),
            hero_sp_attributes: snap.hero_sp_attributes,
            last_change_hero_uid: Some(0),
        }
    }
}
