use crate::state::battle::{
    effects::effect_types::EffectType, mechanics::bloodtithe::BloodtitheState,
    step_builder::FightStepBuilder,
};

use super::super::utils::*;
use anyhow::Result;
use sonettobuf::{ActEffect, Fight, FightEntityInfo};

pub fn build_battle_start(
    entity: &FightEntityInfo,
    fight: &Fight,
    bloodtithe: &mut BloodtitheState,
) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);
    let mut effects = vec![];

    for &passive_id in &entity.passive_skill.clone() {
        match passive_id {
            31250142 => effects.extend(create_overflow_banking(uid)?),
            31250143 => effects.push(create_stat_boost(uid)?),
            31250144 => effects.extend(create_team_healing(entity, fight)?),
            31250151 => effects.extend(create_bloodtithe_buff(entity, fight, bloodtithe)?),
            _ => {}
        }
    }

    Ok(effects)
}

pub fn build_post_power(_entity: &FightEntityInfo, _fight: &Fight) -> Result<Vec<ActEffect>> {
    Ok(vec![])
}

pub fn build_round_start(_entity: &FightEntityInfo, _fight: &Fight) -> Result<Vec<ActEffect>> {
    Ok(vec![])
}

fn create_bloodtithe_buff(
    entity: &FightEntityInfo,
    fight: &Fight,
    _bloodtithe: &mut BloodtitheState,
) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);
    let team_type = entity.team_type.unwrap_or(1);
    let team_uids = get_team_uids(fight, team_type);

    let mut inner_effects = Vec::new();

    for target_uid in team_uids {
        if find_entity_by_uid(fight, target_uid).is_some() {
            inner_effects.push(buff_add(target_uid, uid, 31250151));
            inner_effects.push(ActEffect {
                effect_type: Some(EffectType::SlaveHalo as i32), // 173
                target_id: Some(target_uid),
                ..Default::default()
            });
            inner_effects.push(effect_none(target_uid));
        }
    }

    let effect = FightStepBuilder::new_effect()
        .add_effect_container(0, 0, uid, inner_effects)
        .build_as_protected_skill();

    Ok(vec![effect])
}

fn create_overflow_banking(uid: i64) -> Result<Vec<ActEffect>> {
    let inner_effects = vec![
        buff_add_with_params(uid, uid, 31250161, "806#0"),
        ActEffect {
            effect_type: Some(EffectType::ExPointOverflowBank as i32), // 214
            target_id: Some(uid),
            ..Default::default()
        },
    ];

    let effect = FightStepBuilder::new_skill(uid, uid, 31250142)
        .add_effects(inner_effects)
        .build_as_act_effect();

    Ok(vec![effect])
}

fn create_stat_boost(uid: i64) -> Result<ActEffect> {
    let effect = FightStepBuilder::new_effect()
        .add_battle_container(
            uid,
            31250143,
            vec![
                max_hp_change(uid, 16850, None),
                current_hp_change(uid, 16850),
                max_hp_change(uid, 16850, None),
                current_hp_change(uid, 16850),
                buff_add(uid, uid, 31250171),
                attr_change(uid),
            ],
        )
        .build_as_act_effect();

    Ok(effect)
}

fn create_team_healing(entity: &FightEntityInfo, fight: &Fight) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);
    let team_type = entity.team_type.unwrap_or(1);
    let team_uids = get_team_uids(fight, team_type);

    let mut inner_effects = Vec::new();

    for target_uid in team_uids {
        inner_effects.push(buff_add(target_uid, uid, 31250181));
        inner_effects.push(ActEffect {
            effect_type: Some(EffectType::Cure as i32), // 27
            target_id: Some(target_uid),
            ..Default::default()
        });
    }

    let effect = FightStepBuilder::new_skill(uid, uid, 31250144)
        .add_effects(inner_effects)
        .build_as_act_effect();

    Ok(vec![effect])
}
