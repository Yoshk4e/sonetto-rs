use super::super::utils::*;
use crate::state::battle::mechanics::bloodtithe::BloodtitheState;
use crate::state::battle::step_builder::FightStepBuilder;
use anyhow::Result;
use sonettobuf::{ActEffect, Fight, FightEntityInfo, fight_hurt_info};

pub fn build_battle_start(
    entity: &FightEntityInfo,
    _fight: &Fight,
    bloodtithe: &mut BloodtitheState,
) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);
    let mut effects = vec![];

    for &passive_id in &entity.passive_skill.clone() {
        match passive_id {
            31200145 => effects.push(create_insight_i(uid)?),
            31200146 => effects.extend(create_faith_mechanic(entity, bloodtithe)?),
            31200149 => effects.extend(create_insight_iii(uid)?),
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

fn create_insight_i(uid: i64) -> Result<ActEffect> {
    Ok(FightStepBuilder::new_effect()
        .add_battle_container(
            uid,
            31200145,
            vec![buff_add(uid, uid, 31200145), effect_none(uid)],
        )
        .build_as_act_effect())
}

fn create_faith_mechanic(
    entity: &FightEntityInfo,
    bloodtithe: &mut BloodtitheState,
) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);
    let current_hp = entity.current_hp.unwrap_or(0);
    let max_hp = entity
        .attr
        .as_ref()
        .and_then(|a| a.hp)
        .or_else(|| entity.base_attr.as_ref().and_then(|a| a.hp))
        .unwrap_or(current_hp);

    let hp_loss = (current_hp as f32 * 0.5) as i32;
    let min_hp = (max_hp as f32 * 0.15) as i32;
    let actual_hp_loss = hp_loss.min(current_hp - min_hp).max(0);

    let team = entity.team_type.unwrap_or(1);

    if actual_hp_loss > 0 {
        bloodtithe.on_hp_lost(uid, team, actual_hp_loss);
    }

    let skill_effects = vec![
        buff_add(uid, uid, 31200145),
        effect_none(uid),
        damage(uid, actual_hp_loss),
        hurt_detail(
            uid,
            actual_hp_loss,
            31200146,
            fight_hurt_info::DamageFromType::SkillEffect.into(),
        ),
        moxie_change(uid, 1),
        bloodtithe_value_change(uid, 1, 2),
        buff_add(uid, uid, 31200146),
        effect_none(uid),
    ];

    let ui_effects = vec![
        bloodtithe_max_change(1, 16),
        moxie_change(uid, 1),
        bloodtithe_value_change(uid, 1, 16),
    ];

    // Build SKILL container ActEffect
    let skill_container = FightStepBuilder::new_skill(uid, uid, 31200146)
        .add_effects(skill_effects)
        .build_as_act_effect();

    // Return vector of ActEffects (not wrapped again)
    let mut out = Vec::new();
    out.push(skill_container);
    out.extend(ui_effects);

    Ok(out)
}

fn create_insight_iii(uid: i64) -> Result<Vec<ActEffect>> {
    let battle_container_effect = FightStepBuilder::new_effect()
        .add_battle_container(
            uid,
            31200149,
            vec![buff_add(uid, uid, 31200184), effect_none(uid)],
        )
        .build_as_act_effect();

    Ok(vec![battle_container_effect])
}
