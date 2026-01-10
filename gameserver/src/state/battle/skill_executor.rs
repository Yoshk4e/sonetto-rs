use anyhow::Result;
use data::exceldb;
use sonettobuf::{ActEffect, FightEntityInfo};
use std::collections::HashMap;

use super::effects::effect_types::EffectType;

pub struct SkillExecutor {
    entities: HashMap<i64, FightEntityInfo>,
}

impl SkillExecutor {
    pub fn new(entities: HashMap<i64, FightEntityInfo>) -> Self {
        Self { entities }
    }

    pub fn execute_skill(
        &self,
        caster_uid: i64,
        target_uid: i64,
        skill_id: i32,
    ) -> Result<Vec<ActEffect>> {
        let mut effects = Vec::new();
        let game_data = exceldb::get();

        let skill_data = game_data.skill_effect.iter().find(|s| s.id == skill_id);

        for i in 1..=20 {
            let condition = self.get_condition(skill_id, i);
            let behavior = self.get_behavior(skill_id, i);
            let behavior_target = self.get_behavior_target(skill_id, i);

            if behavior.is_empty() {
                continue;
            }

            if !condition.is_empty() {
                let condition_target = self.get_condition_target(skill_id, i);
                if !self.check_condition(caster_uid, target_uid, &condition, condition_target) {
                    continue;
                }
            }

            let behavior_effects =
                self.execute_behavior(caster_uid, target_uid, &behavior, behavior_target)?;

            effects.extend(behavior_effects);
        }

        if effects.is_empty() {
            if let Some(skill) = skill_data {
                if skill.damage_rate > 0 {
                    tracing::info!(
                        "Using damageRate fallback for skill {}: {}%",
                        skill_id,
                        skill.damage_rate
                    );
                    if let Some(damage_effect) = self.calculate_damage_effect(
                        caster_uid,
                        target_uid,
                        skill.damage_rate,
                        false,
                    ) {
                        effects.push(damage_effect);
                    }
                }
            }
        }

        Ok(effects)
    }

    fn check_condition(
        &self,
        _caster_uid: i64,
        _target_uid: i64,
        _condition: &str,
        condition_target: i32,
    ) -> bool {
        // Self-target conditions pass for now
        if condition_target == 103 {
            return true;
        }
        true
    }

    fn execute_behavior(
        &self,
        caster_uid: i64,
        target_uid: i64,
        behavior: &str,
        behavior_target: i32,
    ) -> Result<Vec<ActEffect>> {
        let game_data = exceldb::get();

        // Parse behavior: "id#param1#param2#param3"
        let parts: Vec<&str> = behavior.split('#').collect();
        if parts.is_empty() {
            return Ok(vec![]);
        }

        let behavior_id: i32 = parts[0].parse().unwrap_or(0);
        let param1: i32 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0);
        let param3: i32 = parts.get(3).and_then(|p| p.parse().ok()).unwrap_or(0);

        let behavior_type = game_data
            .skill_behavior
            .iter()
            .find(|b| b.id == behavior_id)
            .map(|b| b.r#type.as_str())
            .unwrap_or("Unknown");

        let resolved_target = self.resolve_target(caster_uid, target_uid, behavior_target);

        tracing::debug!(
            "Behavior {} ({}): caster={}, target={}, param={}",
            behavior_id,
            behavior_type,
            caster_uid,
            resolved_target,
            param1
        );

        let effects = match behavior_type {
            "Damage" | "Damage2" => {
                if let Some(effect) =
                    self.calculate_damage_effect(caster_uid, resolved_target, param1, false)
                {
                    vec![effect]
                } else {
                    vec![]
                }
            }
            "Heal" | "HealCantCrit" => {
                if let Some(effect) =
                    self.calculate_heal_effect(caster_uid, resolved_target, param1, false)
                {
                    vec![effect]
                } else {
                    vec![]
                }
            }
            "HealByTwoAttr" => {
                // params: #min#max#attr1_percent#?#target#attr2_percent
                let attr1_percent = param3 as f32 / 1000.0; // 250 -> 0.25 (25%)
                let attr2_percent = parts
                    .get(6)
                    .and_then(|p| p.parse::<i32>().ok())
                    .unwrap_or(0) as f32
                    / 1000.0; // 200 -> 0.20 (20%)

                let caster = self.entities.get(&caster_uid);
                let target = self.entities.get(&resolved_target);

                if let (Some(caster), Some(target)) = (caster, target) {
                    // First attribute: target's missing HP
                    let target_current_hp = target.current_hp.unwrap_or(0);
                    let target_max_hp = target
                        .attr
                        .as_ref()
                        .and_then(|a| a.hp)
                        .unwrap_or(target_current_hp);
                    let missing_hp = target_max_hp - target_current_hp;
                    let heal_from_missing = (missing_hp as f32 * attr1_percent) as i32;

                    // Second attribute: caster's max HP
                    let caster_max_hp = caster.attr.as_ref().and_then(|a| a.hp).unwrap_or(1000);
                    let heal_from_caster_hp = (caster_max_hp as f32 * attr2_percent) as i32;

                    let total_heal = heal_from_missing + heal_from_caster_hp;

                    tracing::debug!(
                        "HealByTwoAttr: {}% missing HP ({}) + {}% caster max HP ({}) = {} total heal",
                        attr1_percent * 100.0,
                        heal_from_missing,
                        attr2_percent * 100.0,
                        heal_from_caster_hp,
                        total_heal
                    );

                    vec![self.create_heal_effect(resolved_target, total_heal, false)]
                } else {
                    vec![]
                }
            }
            "AddBuff" | "AddBuffRound" | "AddBuffRound2" | "ConsumeBloodAddBuff" => {
                vec![self.create_buff_effect(caster_uid, resolved_target, param1)]
            }
            "AddExPoint" => {
                vec![self.create_simple_effect(resolved_target, EffectType::AddExPoint, param1)]
            }
            "Purify1" | "Purify2" => {
                vec![self.create_simple_effect(resolved_target, EffectType::Purify, param1)]
            }
            "Bloodlust" => {
                vec![self.create_simple_effect(resolved_target, EffectType::Bloodlust, param1)]
            }
            "AverageLife" => {
                vec![self.create_simple_effect(resolved_target, EffectType::AverageLife, param1)]
            }
            "BloodPoolValueChange" => {
                vec![self.create_simple_effect(
                    resolved_target,
                    EffectType::BloodPoolValueChange,
                    param1,
                )]
            }
            "Detonate" | "Detonate2" => {
                if let Some(effect) =
                    self.calculate_damage_effect(caster_uid, resolved_target, param1, false)
                {
                    vec![effect]
                } else {
                    vec![]
                }
            }

            "AttrFix" | "AttrFixExPoint" | "AttrFixBuff" | "SkillRateUp" | "SkillRateUp1"
            | "SkillRateUp2" | "SkillRateUpExPoint" | "SkillPowerUp" | "RaspberryAddCount" => {
                tracing::debug!("Passive modifier, no immediate effect");
                vec![]
            }
            // Not yet implemented
            "DecrDurationAndCountBuffType"
            | "DecrDurationAndCountBuffTypeId"
            | "DecrDurationAndCountBuffTypeGroup" => {
                tracing::debug!("Buff manipulation not yet implemented");
                vec![]
            }
            "LostLife" => {
                let caster = self.entities.get(&caster_uid);
                if let Some(caster) = caster {
                    let current_hp = caster.current_hp.unwrap_or(0);
                    let hp_lost = (current_hp as f32 * 0.10) as i32;

                    let mut effects = vec![self.create_damage_effect(caster_uid, hp_lost, false)];

                    if let Some(enemy_dmg) = self.calculate_damage_effect(
                        caster_uid,
                        resolved_target,
                        hp_lost * 12 / 10,
                        false,
                    ) {
                        effects.push(enemy_dmg);
                    }
                    effects
                } else {
                    vec![]
                }
            }
            _ => {
                tracing::warn!("Unknown behavior type: {}", behavior_type);
                vec![]
            }
        };

        Ok(effects)
    }

    fn calculate_damage_effect(
        &self,
        caster_uid: i64,
        target_uid: i64,
        base_param: i32,
        is_crit: bool,
    ) -> Option<ActEffect> {
        let caster = self.entities.get(&caster_uid)?;
        let target = self.entities.get(&target_uid)?;

        // Get stats
        let caster_attack = caster.attr.as_ref()?.attack.unwrap_or(100);
        let target_defense = target.attr.as_ref()?.defense.unwrap_or(50);
        let _target_mdefense = target.attr.as_ref()?.mdefense.unwrap_or(50);

        // Theoretical Damage = [Total ATK - Enemy Mental OR Reality DEF * (1 - Penetration Rate) * (1 - DEF Reduction)]
        // * [Critical Damage - Enemy Critical Def]
        // * [1 + DMG Bonus + DMG Boost - Enemy DMG Reduction]
        // * [1 + Incantation Might OR Ultimate Might]
        // * Skill Multiplier
        // * Afflatus Advantage

        // For now, simplified version:
        // damage = base_param (skill multiplier) * (attack - defense * 0.5)

        let penetration_rate = 0.0; // TODO: Get from buffs/stats
        let def_reduction = 0.0; // TODO: Get from buffs

        // Use reality defense for now (could check skill type)
        let effective_defense =
            (target_defense as f32 * (1.0 - penetration_rate) * (1.0 - def_reduction)) as i32;
        let attack_contribution = (caster_attack - effective_defense).max(0);

        // Skill multiplier from base_param (usually a percentage like 1200 = 120%)
        let skill_multiplier = base_param as f32 / 1000.0;

        // Critical damage
        let crit_multiplier = if is_crit { 1.5 } else { 1.0 };

        // DMG bonus/boost (TODO: from buffs)
        let dmg_bonus = 1.0;

        // Incantation/Ultimate might (TODO: from card type)
        let might_bonus = 1.0;

        // Afflatus advantage (TODO: from element matching)
        let afflatus_mult = 1.0;

        let final_damage = (attack_contribution as f32
            * skill_multiplier
            * crit_multiplier
            * dmg_bonus
            * might_bonus
            * afflatus_mult) as i32;

        let final_damage = final_damage.max(1); // Minimum 1 damage

        tracing::debug!(
            "Damage calc: atk={}, def={}, skill_mult={}, final={}",
            caster_attack,
            target_defense,
            skill_multiplier,
            final_damage
        );

        Some(self.create_damage_effect(target_uid, final_damage, is_crit))
    }

    fn calculate_heal_effect(
        &self,
        caster_uid: i64,
        target_uid: i64,
        base_param: i32,
        is_crit: bool,
    ) -> Option<ActEffect> {
        let caster = self.entities.get(&caster_uid)?;

        // Healing often scales with caster's attack or a specific stat
        let caster_attack = caster.attr.as_ref()?.attack.unwrap_or(100);

        // Heal formula: base + (attack * multiplier)
        let heal_contribution = (caster_attack as f32 * (base_param as f32 / 100.0)) as i32;
        let final_heal = (base_param + heal_contribution).max(1);

        tracing::debug!(
            "Heal calc: base={}, atk={}, final={}",
            base_param,
            caster_attack,
            final_heal
        );

        Some(self.create_heal_effect(target_uid, final_heal, is_crit))
    }

    fn resolve_target(&self, caster_uid: i64, target_uid: i64, behavior_target: i32) -> i64 {
        match behavior_target {
            103 => caster_uid, // Self - buff yourself
            102 => caster_uid, // Self
            101 => caster_uid, // Random ally (for now, use caster)
            1 => target_uid,   // Single target (use skill target)
            2 => target_uid,   // Random enemy (use skill target)
            201 => target_uid, // All enemies (for now, use skill target)
            202 => target_uid, // Random enemies
            _ => {
                tracing::warn!(
                    "Unknown behavior_target: {}, using skill target",
                    behavior_target
                );
                target_uid
            }
        }
    }

    // Helper to create simple effects
    fn create_simple_effect(
        &self,
        target_id: i64,
        effect_type: EffectType,
        value: i32,
    ) -> ActEffect {
        ActEffect {
            target_id: Some(target_id),
            effect_type: Some(effect_type.to_i32()),
            effect_num: Some(value),
            buff: None,
            entity: None,
            config_effect: Some(0),
            buff_act_id: Some(0),
            reserve_id: Some(0),
            reserve_str: Some(String::new()),
            summoned: None,
            magic_circle: None,
            card_info: None,
            card_info_list: vec![],
            team_type: Some(0),
            fight_step: None,
            assist_boss_info: None,
            effect_num1: Some(0),
            emitter_info: None,
            player_finisher_info: None,
            power_info: None,
            card_heat_value: None,
            fight_tasks: vec![],
            fight: None,
            buff_act_info: None,
            hurt_info: None,
        }
    }

    fn create_damage_effect(&self, target_id: i64, damage: i32, is_crit: bool) -> ActEffect {
        let effect_type = if is_crit {
            EffectType::Crit
        } else {
            EffectType::Damage
        };

        self.create_simple_effect(target_id, effect_type, damage)
    }

    fn create_heal_effect(&self, target_id: i64, heal: i32, is_crit: bool) -> ActEffect {
        let effect_type = if is_crit {
            EffectType::HealCrit
        } else {
            EffectType::Heal
        };

        self.create_simple_effect(target_id, effect_type, heal)
    }

    fn create_buff_effect(&self, caster_uid: i64, target_id: i64, buff_id: i32) -> ActEffect {
        ActEffect {
            target_id: Some(target_id),
            effect_type: Some(EffectType::BuffAdd.to_i32()),
            effect_num: Some(buff_id),
            buff: Some(sonettobuf::BuffInfo {
                buff_id: Some(buff_id),
                duration: Some(0), // TODO: Load from buff table
                uid: Some(0),      // TODO: Generate unique UID
                ex_info: Some(0),
                from_uid: Some(caster_uid),
                count: Some(0),
                layer: Some(1),
                r#type: Some(0),
                act_common_params: Some(String::new()),
                act_info: vec![],
            }),
            entity: None,
            config_effect: Some(0),
            buff_act_id: Some(0),
            reserve_id: Some(0),
            reserve_str: Some(String::new()),
            summoned: None,
            magic_circle: None,
            card_info: None,
            card_info_list: vec![],
            team_type: Some(0),
            fight_step: None,
            assist_boss_info: None,
            effect_num1: Some(0),
            emitter_info: None,
            player_finisher_info: None,
            power_info: None,
            card_heat_value: None,
            fight_tasks: vec![],
            fight: None,
            buff_act_info: None,
            hurt_info: None,
        }
    }

    fn get_condition(&self, skill_id: i32, index: i32) -> String {
        let game_data = exceldb::get();
        let skill = game_data.skill_effect.iter().find(|s| s.id == skill_id);
        let Some(skill) = skill else {
            return String::new();
        };

        match index {
            1 => skill.condition1.clone(),
            2 => skill.condition2.clone(),
            3 => skill.condition3.clone(),
            4 => skill.condition4.clone(),
            5 => skill.condition5.clone(),
            6 => skill.condition6.clone(),
            7 => skill.condition7.clone(),
            8 => skill.condition8.clone(),
            9 => skill.condition9.clone(),
            10 => skill.condition10.clone(),
            11 => skill.condition11.clone(),
            12 => skill.condition12.clone(),
            13 => skill.condition13.clone(),
            14 => skill.condition14.clone(),
            15 => skill.condition15.clone(),
            16 => skill.condition16.clone(),
            17 => skill.condition17.clone(),
            18 => skill.condition18.clone(),
            19 => skill.condition19.clone(),
            20 => skill.condition20.clone(),
            _ => String::new(),
        }
    }

    fn get_condition_target(&self, skill_id: i32, index: i32) -> i32 {
        let game_data = exceldb::get();
        let skill = game_data.skill_effect.iter().find(|s| s.id == skill_id);
        let Some(skill) = skill else { return 0 };

        match index {
            1 => skill.condition_target1,
            2 => skill.condition_target2,
            3 => skill.condition_target3,
            4 => skill.condition_target4,
            5 => skill.condition_target5,
            6 => skill.condition_target6,
            7 => skill.condition_target7,
            8 => skill.condition_target8,
            9 => skill.condition_target9,
            10 => skill.condition_target10,
            11 => skill.condition_target11,
            12 => skill.condition_target12,
            13 => skill.condition_target13,
            14 => skill.condition_target14,
            15 => skill.condition_target15,
            16 => skill.condition_target16,
            17 => skill.condition_target17,
            18 => skill.condition_target18,
            19 => skill.condition_target19,
            20 => skill.condition_target20,
            _ => 0,
        }
    }

    fn get_behavior(&self, skill_id: i32, index: i32) -> String {
        let game_data = exceldb::get();
        let skill = game_data.skill_effect.iter().find(|s| s.id == skill_id);
        let Some(skill) = skill else {
            return String::new();
        };

        match index {
            1 => skill.behavior1.clone(),
            2 => skill.behavior2.clone(),
            3 => skill.behavior3.clone(),
            4 => skill.behavior4.clone(),
            5 => skill.behavior5.clone(),
            6 => skill.behavior6.clone(),
            7 => skill.behavior7.clone(),
            8 => skill.behavior8.clone(),
            9 => skill.behavior9.clone(),
            10 => skill.behavior10.clone(),
            11 => skill.behavior11.clone(),
            12 => skill.behavior12.clone(),
            13 => skill.behavior13.clone(),
            14 => skill.behavior14.clone(),
            15 => skill.behavior15.clone(),
            16 => skill.behavior16.clone(),
            17 => skill.behavior17.clone(),
            18 => skill.behavior18.clone(),
            19 => skill.behavior19.clone(),
            20 => skill.behavior20.clone(),
            _ => String::new(),
        }
    }

    fn get_behavior_target(&self, skill_id: i32, index: i32) -> i32 {
        let game_data = exceldb::get();
        let skill = game_data.skill_effect.iter().find(|s| s.id == skill_id);
        let Some(skill) = skill else { return 0 };

        match index {
            1 => skill.behavior_target1,
            2 => skill.behavior_target2,
            3 => skill.behavior_target3,
            4 => skill.behavior_target4,
            5 => skill.behavior_target5,
            6 => skill.behavior_target6,
            7 => skill.behavior_target7,
            8 => skill.behavior_target8,
            9 => skill.behavior_target9,
            10 => skill.behavior_target10,
            11 => skill.behavior_target11,
            12 => skill.behavior_target12,
            13 => skill.behavior_target13,
            14 => skill.behavior_target14,
            15 => skill.behavior_target15,
            16 => skill.behavior_target16,
            17 => skill.behavior_target17,
            18 => skill.behavior_target18,
            19 => skill.behavior_target19,
            20 => skill.behavior_target20,
            _ => 0,
        }
    }
}
