use super::BUFF_UID_COUNTER;
use sonettobuf::{
    ActEffect, BuffInfo, Fight, FightEntityInfo, FightHurtInfo, effect_type_enum::EffectType,
};
use std::sync::atomic::Ordering;

//skill_behaviour table
pub enum VfxConfig {
    Heal = 20001,
    Damage = 30006,
    Moxie = 20002,
}

pub fn buff_add(target_uid: i64, from_uid: i64, buff_id: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Buffadd as i32),
        target_id: Some(target_uid),
        effect_num: Some(buff_id),
        buff: Some(create_buff(buff_id, from_uid, 0, "")),
        ..Default::default()
    }
}

#[allow(dead_code)]
pub fn buff_add_with_count(target_uid: i64, buff_id: i32, count: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Buffadd as i32),
        target_id: Some(target_uid),
        effect_num: Some(buff_id),
        buff: Some(create_buff(buff_id, target_uid, count, "")),
        ..Default::default()
    }
}

pub fn buff_add_with_params(
    target_uid: i64,
    from_uid: i64,
    buff_id: i32,
    params: &str,
) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Buffadd as i32),
        target_id: Some(target_uid),
        effect_num: Some(buff_id),
        buff: Some(create_buff(buff_id, from_uid, 0, params)),
        ..Default::default()
    }
}

/// Damage with visual effect
pub fn damage(target_uid: i64, amount: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Damage as i32),
        target_id: Some(target_uid),
        effect_num: Some(amount),
        config_effect: Some(VfxConfig::Damage as i32),
        ..Default::default()
    }
}

#[allow(dead_code)]
pub fn damage_buff(target_uid: i64, amount: i32, buff_id: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Damage as i32),
        target_id: Some(target_uid),
        effect_num: Some(amount),
        buff_act_id: Some(buff_id),
        ..Default::default()
    }
}

/// Damage details
pub fn hurt_detail(target_uid: i64, damage: i32, skill_id: i32, damage_type: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Fighthurtdetail as i32),
        target_id: Some(target_uid),
        hurt_info: Some(FightHurtInfo {
            damage: Some(damage),
            reduce_hp: Some(damage),
            reduce_shield: Some(0),
            career_restraint: Some(false),
            critical: Some(false),
            assassinate: Some(false),
            hurt_effect: Some(EffectType::Damage as i32),
            damage_from_type: Some(damage_type),
            config_effect: Some(VfxConfig::Damage as i32),
            effect_id: Some(skill_id),
            skill_id: Some(skill_id),
            from_uid: Some(target_uid),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[allow(dead_code)]
pub fn hurt_detail_buff(
    target_uid: i64,
    damage: i32,
    skill_id: i32,
    damage_type: i32,
    buff_id: i32,
) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Fighthurtdetail as i32),
        target_id: Some(target_uid),
        hurt_info: Some(FightHurtInfo {
            damage: Some(damage),
            reduce_hp: Some(damage),
            reduce_shield: Some(0),
            career_restraint: Some(false),
            critical: Some(false),
            assassinate: Some(false),
            hurt_effect: Some(EffectType::Damage as i32),
            damage_from_type: Some(damage_type),
            config_effect: Some(0),
            buff_act_id: Some(buff_id),
            effect_id: Some(skill_id),
            skill_id: Some(skill_id),
            from_uid: Some(target_uid),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Moxie change with visual effect
pub fn moxie_change(target_uid: i64, amount: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Expointchange as i32),
        target_id: Some(target_uid),
        effect_num: Some(amount),
        config_effect: Some(VfxConfig::Moxie as i32),
        ..Default::default()
    }
}

/// Bloodtithe max pool change
pub fn bloodtithe_max_change(amount: i32, change_type: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Bloodpoolmaxchange as i32),
        target_id: Some(0),
        effect_num: Some(amount),
        effect_num1: Some(change_type),
        ..Default::default()
    }
}

/// Bloodtithe value change
pub fn bloodtithe_value_change(target_uid: i64, amount: i32, change_type: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Bloodpoolvaluechange as i32),
        target_id: Some(target_uid),
        effect_num: Some(amount),
        effect_num1: Some(change_type),
        ..Default::default()
    }
}

/// Changes your max hp not current
pub fn max_hp_change(target_uid: i64, amount: i32, buff_id: Option<i32>) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Maxhpchange as i32),
        target_id: Some(target_uid),
        effect_num: Some(amount),
        buff_act_id: buff_id,
        ..Default::default()
    }
}

/// Changes your current hp not max
pub fn current_hp_change(target_uid: i64, amount: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Currenthpchange as i32),
        target_id: Some(target_uid),
        effect_num: Some(amount),
        ..Default::default()
    }
}

pub fn attr_change(target_uid: i64) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Attr as i32),
        target_id: Some(target_uid),
        ..Default::default()
    }
}

pub fn effect_none(target_uid: i64) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::None as i32),
        target_id: Some(target_uid),
        ..Default::default()
    }
}

fn create_buff(buff_id: i32, from_uid: i64, count: i32, params: &str) -> BuffInfo {
    BuffInfo {
        buff_id: Some(buff_id),
        duration: Some(0),
        uid: Some(BUFF_UID_COUNTER.fetch_add(1, Ordering::SeqCst)),
        ex_info: Some(0),
        from_uid: Some(from_uid),
        count: Some(count),
        act_common_params: Some(params.to_string()),
        layer: Some(0),
        r#type: Some(0),
        act_info: vec![],
    }
}

pub fn get_team_uids(fight: &Fight, team_type: i32) -> Vec<i64> {
    let team = if team_type == 1 {
        &fight.attacker
    } else {
        &fight.defender
    };

    team.as_ref()
        .map(|t| t.entitys.iter().filter_map(|e| e.uid).collect())
        .unwrap_or_default()
}

pub fn find_entity_by_uid(fight: &Fight, uid: i64) -> Option<&FightEntityInfo> {
    fight
        .attacker
        .as_ref()
        .and_then(|attacker| attacker.entitys.iter().find(|e| e.uid == Some(uid)))
        .or_else(|| {
            fight
                .defender
                .as_ref()
                .and_then(|defender| defender.entitys.iter().find(|e| e.uid == Some(uid)))
        })
}
