use crate::state::battle::{
    effects::effect_types::EffectType, mechanics::bloodtithe::BloodtitheState, step_builder::FightStepBuilder
};

use super::super::utils::*;
use anyhow::Result;
use sonettobuf::{ActEffect, Fight, FightEntityInfo, fight_hurt_info};

pub fn build_battle_start(
    entity: &FightEntityInfo,
    fight: &Fight,
    bloodtithe: &mut BloodtitheState,
) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);
    let mut effects = vec![];

    for &passive_id in &entity.passive_skill.clone() {
        match passive_id {
            30880141 => effects.extend(create_bloodtithe_buff(entity, fight, bloodtithe)?),
            30880151 => effects.push(create_counter(uid)?),
            _ => {}
        }
    }

    Ok(effects)
}

pub fn build_post_power(entity: &FightEntityInfo, _fight: &Fight) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);

    if entity.passive_skill.contains(&308802111) {
        let effect = FightStepBuilder::new_effect()
            .add_battle_container(
                uid,
                308802111,
                vec![ActEffect {
                    effect_type: Some(EffectType::None as i32),
                    target_id: Some(uid),
                    effect_num: Some(308802111),
                    buff_act_id: Some(1021),
                    ..Default::default()
                }],
            )
            .build_as_act_effect();

        Ok(vec![effect])
    } else {
        Ok(vec![])
    }
}

pub fn build_round_start(
    _entity: &FightEntityInfo,
    _fight: &Fight,
    _bloodtithe: &mut BloodtitheState,
) -> Result<Vec<ActEffect>> {
    Ok(vec![])
}

fn create_bloodtithe_buff(
    entity: &FightEntityInfo,
    fight: &Fight,
    bloodtithe: &mut BloodtitheState,
) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);
    let team = entity.team_type.unwrap_or(1);

    let max_hp = entity
        .attr
        .as_ref()
        .and_then(|a| a.hp)
        .unwrap_or(entity.current_hp.unwrap_or(0));

    let current_hp = entity.current_hp.unwrap_or(max_hp);

    // +50% max HP
    let hp_bonus = ((max_hp as f32) * 0.50).round() as i32;
    let new_max_hp = max_hp + hp_bonus;

    // Lose 20% of current HP
    let hp_loss = ((new_max_hp as f32) * 0.20).round() as i32;
    let new_current_hp = (new_max_hp - hp_loss).max(1);

    if hp_loss > 0 {
        bloodtithe.on_hp_lost(uid, team, hp_loss);
    }

    tracing::info!(
        "HP Sacrifice: uid={}, max_hp {} -> {}, current_hp {} -> {}, bonus={}, loss={}",
        uid,
        max_hp,
        new_max_hp,
        current_hp,
        new_current_hp,
        hp_bonus,
        hp_loss
    );

    if entity.passive_skill.contains(&30880141) {
        let mut effects = vec![];

        // Create bloodtithe buff effect
        let bloodtithe_effect = FightStepBuilder::new_effect()
            .add_battle_container(
                uid,
                30880141,
                vec![
                    moxie_change(uid, 3),
                    damage(uid, hp_loss),
                    hurt_detail(
                        uid,
                        hp_loss,
                        30880141,
                        fight_hurt_info::DamageFromType::SkillEffect.into(),
                    ),
                    moxie_change(uid, 1),
                    bloodtithe_value_change(uid, 1, 1),
                ],
            )
            .build_as_act_effect();

        effects.push(bloodtithe_effect);

        // Add team buffs
        effects.extend(create_team_buffs(entity, fight)?);

        Ok(effects)
    } else {
        Ok(vec![])
    }
}

fn create_team_buffs(entity: &FightEntityInfo, fight: &Fight) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);
    let team_type = entity.team_type.unwrap_or(1);
    let team_uids = get_team_uids(fight, team_type);

    let max_hp = entity
        .attr
        .as_ref()
        .and_then(|a| a.hp)
        .unwrap_or(entity.current_hp.unwrap_or(0));

    // +50% max HP
    let hp_bonus = ((max_hp as f32) * 0.50).round() as i32;
    let new_max_hp = max_hp + hp_bonus;
    let share = ((hp_bonus as f32) * 0.10).round() as i32;

    // Lose 20% of current HP
    let hp_loss = ((new_max_hp as f32) * 0.20).round() as i32;
    let new_current_hp = (new_max_hp - hp_loss).max(1);

    tracing::info!(
        "Team HP share: uid={} bonus={} share_per_ally={}",
        uid,
        hp_bonus,
        share
    );

    let mut inner: Vec<ActEffect> = Vec::new();

    for target_uid in team_uids {
        if target_uid == uid {
            // Self buffs
            inner.push(max_hp_change(uid, new_max_hp, None));
            inner.push(current_hp_change(uid, new_current_hp));
            inner.push(max_hp_change(uid, new_max_hp, None));
            inner.push(current_hp_change(uid, new_current_hp));
            inner.push(buff_add(uid, uid, 308801921));
            inner.push(attr_change(uid));
            continue;
        }

        if let Some(target) = find_entity_by_uid(fight, target_uid).as_mut() {
            let cur_hp = target.current_hp.unwrap_or(0);
            let target_max_hp = target.attr.as_ref().and_then(|a| a.hp).unwrap_or(cur_hp);

            let new_cur = cur_hp + share;
            let new_max = target_max_hp + share;

            // Ally buffs
            inner.push(max_hp_change(target_uid, new_max, None));
            inner.push(current_hp_change(target_uid, new_cur));
            inner.push(buff_add(target_uid, uid, 308801922));
            inner.push(effect_none(target_uid));
        }
    }

    if inner.is_empty() {
        return Ok(vec![]);
    }

    let team_buff_effect = FightStepBuilder::new_effect()
        .add_battle_container(uid, 308801921, inner)
        .build_as_act_effect();

    Ok(vec![team_buff_effect])
}

fn create_counter(uid: i64) -> Result<ActEffect> {
    let effect = FightStepBuilder::new_effect()
        .add_battle_container(
            uid,
            30880151,
            vec![buff_add(uid, uid, 205), attr_change(uid)],
        )
        .build_as_act_effect();

    Ok(effect)
}
