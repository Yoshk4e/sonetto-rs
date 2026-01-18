use crate::state::battle::step_builder::FightStepBuilder;

use super::super::utils::*;
use anyhow::Result;
use sonettobuf::{ActEffect, FightEntityInfo};

pub fn build_battle_start(entity: &FightEntityInfo) -> Result<Vec<ActEffect>> {
    let model_id = entity.model_id.unwrap_or(0);
    let uid = entity.uid.unwrap_or(0);
    let mut effects = Vec::new();

    match model_id {
        3088 => {
            let activity_buff = FightStepBuilder::new_skill(uid, uid, 308801911)
                .add_effect(buff_add(uid, uid, 6270501))
                .add_effect(effect_none(uid))
                .build_as_act_effect();
            effects.push(activity_buff);

            let base_buff = FightStepBuilder::new_skill(uid, uid, 308802111)
                .add_effect(buff_add(uid, uid, 308802111))
                .add_effect(effect_none(uid))
                .build_as_act_effect();
            effects.push(base_buff);
        }

        3120 => {
            let skill = FightStepBuilder::new_skill(uid, uid, 31200146)
                .add_effect(buff_add(uid, uid, 6270501))
                .add_effect(effect_none(uid))
                .build_as_protected_skill();
            effects.push(skill);
        }

        3125 => {
            let base_buff = FightStepBuilder::new_skill(uid, uid, 31250141)
                .add_effect(buff_add(uid, uid, 31250131))
                .add_effect(effect_none(uid))
                .build_as_act_effect();
            effects.push(base_buff);
        }

        _ => {}
    }

    Ok(effects)
}
