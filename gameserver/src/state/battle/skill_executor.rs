use anyhow::Result;
use data::exceldb;
use sonettobuf::effect_type_enum::EffectType;
use sonettobuf::{
    ActEffect, BuffInfo, FightEntityInfo, FightHurtInfo, FightStep, fight_hurt_info, fight_step,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::state::battle::manager::buff_mgr::BuffMgr;

use super::utils::VfxConfig;

static BUFF_UID_COUNTER: AtomicI64 = AtomicI64::new(1000);

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
        buff_mgr: &BuffMgr,
    ) -> Result<FightStep> {
        let mut effects = Vec::new();
        let game_data = exceldb::get();

        let skill_data = game_data.skill_effect.iter().find(|s| s.id == skill_id);

        tracing::warn!("=== EXECUTING SKILL {} ===", skill_id);

        let mut any_behavior_defined = false;

        // Process all 20 behavior slots
        for i in 1..=20 {
            let condition = self.get_condition(skill_id, i);
            let behavior = self.get_behavior(skill_id, i);
            if !behavior.is_empty() {
                any_behavior_defined = true;
            }

            if behavior.is_empty() {
                continue;
            }

            let behavior_target = self.get_behavior_target(skill_id, i);
            let condition_target = self.get_condition_target(skill_id, i);
            // 999 = inherit condition target (passive semantics)
            let effective_target = if behavior_target == 999 {
                condition_target
            } else {
                behavior_target
            };

            tracing::warn!(
                "Behavior slot {}: behavior='{}', target={} (effective={})",
                i,
                behavior,
                behavior_target,
                effective_target
            );

            // Check condition if present
            if !condition.is_empty() {
                let condition_target = self.get_condition_target(skill_id, i);
                if !self.check_condition(caster_uid, &condition, condition_target, buff_mgr) {
                    continue;
                }
            }

            // Execute behavior
            let behavior_effects =
                self.execute_behavior(caster_uid, target_uid, &behavior, effective_target)?;
            effects.extend(behavior_effects);
        }

        // Fallback to damageRate if no behaviors produced effects
        if !any_behavior_defined
            && let Some(skill) = skill_data
            && skill.damage_rate > 0
        {
            tracing::info!(
                "Using damageRate fallback for skill {}: {}%",
                skill_id,
                skill.damage_rate
            );
            if let Some(damage_effect) =
                self.calculate_damage_effect(caster_uid, target_uid, skill.damage_rate, false)
            {
                effects.push(damage_effect);
            }
        }

        Ok(FightStep {
            act_type: Some(fight_step::ActType::Skill.into()),
            from_id: Some(caster_uid),
            to_id: Some(target_uid),
            act_id: Some(skill_id),
            act_effect: effects,
            card_index: Some(0),
            support_hero_id: Some(0),
            fake_timeline: Some(false),
        })
    }

    fn check_condition(
        &self,
        caster_uid: i64,
        condition: &str,
        condition_target: i32,
        buff_mgr: &BuffMgr,
    ) -> bool {
        if condition.is_empty() {
            return true;
        }

        let parts: Vec<&str> = condition.split('#').collect();
        let cond_id: i32 = parts[0].parse().unwrap_or(0);

        match cond_id {
            203 => true,
            208 => true, // always true

            57210 => {
                // has buff
                let buff_id: i32 = parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);

                match condition_target {
                    101 | 103 => {
                        // all allies must have buff
                        self.entities
                            .values()
                            .filter(|e| e.team_type == self.entities[&caster_uid].team_type)
                            .all(|e| buff_mgr.has_buff(e.uid.unwrap(), buff_id))
                    }
                    _ => false,
                }
            }

            19210 => {
                // does NOT have buff
                let buff_id: i32 = parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                !buff_mgr.has_buff(caster_uid, buff_id)
            }

            _ => {
                tracing::warn!("Unknown condition: {}", condition);
                false
            }
        }
    }

    fn execute_behavior(
        &self,
        caster_uid: i64,
        target_uid: i64,
        behavior: &str,
        behavior_target: i32,
    ) -> Result<Vec<ActEffect>> {
        let game_data = exceldb::get();

        // Parse behavior: "id#param1#param2#param3..."
        let parts: Vec<&str> = behavior.split('#').collect();
        if parts.is_empty() {
            return Ok(vec![]);
        }

        let behavior_id: i32 = parts[0].parse().unwrap_or(0);
        let param1: i32 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0);
        let param2: i32 = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);

        let behavior_type = game_data
            .skill_behavior
            .iter()
            .find(|b| b.id == behavior_id)
            .map(|b| b.r#type.as_str())
            .unwrap_or("Unknown");

        let mut targets = self.resolve_targets(caster_uid, target_uid, behavior_target);

        tracing::warn!(
            "Behavior {} ({}): caster={}, targets={:?}, p1={}, p2={}",
            behavior_id,
            behavior_type,
            caster_uid,
            targets,
            param1,
            param2
        );

        match behavior_type {
            "LostLife" | "Bloodlust" | "RaspberryAddCount" | "AttrFix" | "AttrFixExPoint"
            | "AttrFixBuff" | "SkillRateUp" | "SkillRateUp1" | "SkillRateUp2"
            | "SkillRateUpExPoint" | "SkillPowerUp" => {
                targets = vec![caster_uid];
            }

            _ => {}
        }

        let mut effects = Vec::new();

        for target in targets {
            match behavior_type {
                "Damage" | "Damage2" | "Detonate" | "Detonate2" => {
                    if let Some(effect) =
                        self.calculate_damage_effect(caster_uid, target, param1, false)
                    {
                        effects.push(effect);
                    }
                }

                "Heal" | "HealCantCrit" => {
                    if let Some(effect) =
                        self.calculate_heal_effect(caster_uid, target, param1, false)
                    {
                        effects.push(effect);
                    }
                }

                "HealByTwoAttr" => {
                    effects.extend(self.execute_heal_by_two_attr(caster_uid, target, &parts)?);
                }

                "AddBuff"
                | "AddBuffRound"
                | "AddBuffRound2"
                | "ConsumeBloodAddBuff"
                | "CreateAdditionalDamageAddBuff" => {
                    effects.push(self.create_buff_effect(caster_uid, target, param1));
                }

                "AddExPoint" => {
                    effects.push(ActEffect {
                        effect_type: Some(EffectType::Addexpoint as i32),
                        target_id: Some(target),
                        effect_num: Some(param1),
                        ..Default::default()
                    });
                }

                "Purify1" | "Purify2" => {
                    effects.push(ActEffect {
                        effect_type: Some(EffectType::Purify as i32),
                        target_id: Some(target),
                        effect_num: Some(param1),
                        ..Default::default()
                    });
                }

                "Bloodlust" => {
                    effects.push(ActEffect {
                        effect_type: Some(EffectType::Bloodlust as i32),
                        target_id: Some(target),
                        effect_num: Some(param1),
                        ..Default::default()
                    });
                }

                "AverageLife" => {
                    effects.push(ActEffect {
                        effect_type: Some(EffectType::Averagelife as i32),
                        target_id: Some(target),
                        ..Default::default()
                    });
                }

                "BloodPoolValueChange" => {
                    effects.push(ActEffect {
                        effect_type: Some(EffectType::Bloodpoolvaluechange as i32),
                        target_id: Some(target),
                        effect_num: Some(param1),
                        effect_num1: Some(param2),
                        ..Default::default()
                    });
                }

                "LostLife" => {
                    effects.extend(self.execute_lost_life(caster_uid, target, &parts)?);
                }

                // Passive-only behaviors (no immediate effects)
                "AttrFix" | "AttrFixExPoint" | "AttrFixBuff" | "SkillRateUp" | "SkillRateUp1"
                | "SkillRateUp2" | "SkillRateUpExPoint" | "SkillPowerUp" | "RaspberryAddCount" => {
                    tracing::warn!("Passive modifier behavior: {}", behavior_type);
                }

                // Not implemented yet
                "DecrDurationAndCountBuffType"
                | "DecrDurationAndCountBuffTypeId"
                | "DecrDurationAndCountBuffTypeGroup" => {
                    tracing::warn!("Buff manipulation not implemented: {}", behavior_type);
                }

                _ => {
                    tracing::warn!("Unknown behavior type: {}", behavior_type);
                }
            }
        }

        Ok(effects)
    }

    fn execute_heal_by_two_attr(
        &self,
        caster_uid: i64,
        target_uid: i64,
        parts: &[&str],
    ) -> Result<Vec<ActEffect>> {
        // params: #min#max#attr1_percent#?#target#attr2_percent
        let attr1_percent = parts
            .get(3)
            .and_then(|p| p.parse::<i32>().ok())
            .unwrap_or(0) as f32
            / 1000.0;
        let attr2_percent = parts
            .get(6)
            .and_then(|p| p.parse::<i32>().ok())
            .unwrap_or(0) as f32
            / 1000.0;

        let caster = self.entities.get(&caster_uid);
        let target = self.entities.get(&target_uid);

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

            tracing::info!(
                "HealByTwoAttr: {}% missing HP ({}) + {}% caster max HP ({}) = {}",
                attr1_percent * 100.0,
                heal_from_missing,
                attr2_percent * 100.0,
                heal_from_caster_hp,
                total_heal
            );

            Ok(vec![self.create_heal_effect(target_uid, total_heal, false)])
        } else {
            Ok(vec![])
        }
    }

    fn execute_lost_life(
        &self,
        _caster_uid: i64,
        target_uid: i64,
        parts: &[&str],
    ) -> Result<Vec<ActEffect>> {
        let percent = parts
            .get(2)
            .and_then(|p| p.parse::<i32>().ok())
            .unwrap_or(0);

        let target = self.entities.get(&target_uid);
        let max_hp = target
            .as_ref()
            .and_then(|t| {
                t.attr
                    .as_ref()
                    .map(|a| a.hp.unwrap_or(t.current_hp.unwrap_or(0)))
            })
            .unwrap_or(0);
        let loss = max_hp * percent / 1000;

        Ok(vec![ActEffect {
            effect_type: Some(EffectType::Currenthpchange as i32),
            target_id: Some(target_uid),
            effect_num: Some(-loss),
            ..Default::default()
        }])
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

        // Simplified damage calculation
        // TODO: Implement full formula with penetration, def reduction, dmg bonus, etc.
        let penetration_rate = 0.0;
        let def_reduction = 0.0;
        let effective_defense =
            (target_defense as f32 * (1.0 - penetration_rate) * (1.0 - def_reduction)) as i32;
        let attack_contribution = (caster_attack - effective_defense).max(0);

        let skill_multiplier = base_param as f32 / 1000.0;
        let crit_multiplier = if is_crit { 1.5 } else { 1.0 };

        let final_damage = (attack_contribution as f32 * skill_multiplier * crit_multiplier) as i32;
        let final_damage = final_damage.max(1);

        tracing::debug!(
            "Damage calc: atk={}, def={}, mult={}, final={}",
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
        let caster_attack = caster.attr.as_ref()?.attack.unwrap_or(100);

        // Simplified heal calculation
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

    fn resolve_targets(&self, caster_uid: i64, target_uid: i64, behavior_target: i32) -> Vec<i64> {
        match behavior_target {
            // Self
            0 | 102 => vec![caster_uid],

            // Explicit selected target (card target)
            1 | 2 => vec![target_uid],

            // All allies (including self)
            101 | 103 => self
                .entities
                .values()
                .filter(|e| e.team_type == self.entities[&caster_uid].team_type)
                .filter_map(|e| e.uid)
                .collect(),

            // All enemies
            201 | 202 => self
                .entities
                .values()
                .filter(|e| e.team_type != self.entities[&caster_uid].team_type)
                .filter_map(|e| e.uid)
                .collect(),

            999 => {
                if target_uid != 0 && self.entities.contains_key(&target_uid) {
                    vec![target_uid]
                } else {
                    tracing::warn!("behavior_target=999 but no valid target; defaulting to self");
                    vec![caster_uid]
                }
            }

            _ => {
                tracing::warn!(
                    "Unknown behavior_target {}, defaulting to caster",
                    behavior_target
                );
                vec![caster_uid]
            }
        }
    }

    fn create_damage_effect(&self, target_id: i64, damage: i32, is_crit: bool) -> ActEffect {
        ActEffect {
            effect_type: Some(if is_crit {
                EffectType::Crit as i32
            } else {
                EffectType::Damage as i32
            }), // CRIT or DAMAGE
            target_id: Some(target_id),
            effect_num: Some(damage),
            config_effect: Some(VfxConfig::Damage as i32), // Standard damage VFX
            hurt_info: Some(FightHurtInfo {
                damage: Some(damage),
                reduce_hp: Some(damage),
                reduce_shield: Some(0),
                career_restraint: Some(false),
                critical: Some(is_crit),
                assassinate: Some(false),
                hurt_effect: Some(if is_crit {
                    EffectType::Crit as i32
                } else {
                    EffectType::Damage as i32
                }),
                damage_from_type: Some(fight_hurt_info::DamageFromType::SkillEffect.into()),
                config_effect: Some(VfxConfig::Damage as i32),
                effect_id: Some(0),
                skill_id: Some(0),
                from_uid: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn create_heal_effect(&self, target_id: i64, heal: i32, is_crit: bool) -> ActEffect {
        ActEffect {
            effect_type: Some(if is_crit {
                EffectType::Healcrit as i32
            } else {
                EffectType::Heal as i32
            }), // HEALCRIT or HEAL
            target_id: Some(target_id),
            effect_num: Some(heal),
            config_effect: Some(VfxConfig::Heal as i32), // Heal VFX
            ..Default::default()
        }
    }

    fn create_buff_effect(&self, caster_uid: i64, target_id: i64, buff_id: i32) -> ActEffect {
        let duration = data::exceldb::get()
            .skill_buff
            .iter()
            .find(|b| b.id == buff_id)
            .map(|b| b.during_time)
            .unwrap_or(0);

        ActEffect {
            effect_type: Some(EffectType::Buffadd as i32),
            target_id: Some(target_id),
            effect_num: Some(buff_id),
            buff: Some(BuffInfo {
                buff_id: Some(buff_id),
                duration: Some(duration),
                uid: Some(BUFF_UID_COUNTER.fetch_add(1, Ordering::SeqCst)),
                ex_info: Some(0),
                from_uid: Some(caster_uid),
                count: Some(0),
                layer: Some(1),
                r#type: Some(0),
                act_common_params: Some(String::new()),
                act_info: vec![],
            }),
            ..Default::default()
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
