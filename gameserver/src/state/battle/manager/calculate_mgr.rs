use sonettobuf::{
    ActEffect, Fight, FightExPointInfo, FightHeroSpAttributeInfo, FightStep, HeroSpAttribute,
    PlayerSkillInfo,
};
use std::sync::Arc;

use crate::state::battle::{
    effects::effect_types::EffectType,
    manager::{
        buff_mgr::BuffMgr,
        entity_mgr::{FightEntityDataMgr, get_entity_mut_by_location},
    },
    mechanics::bloodtithe::BloodtitheState,
};

#[derive(Default, Debug, Clone)]
pub struct FightCalculateDataMgr {
    fight: Arc<Fight>,
    entity_mgr: FightEntityDataMgr,
    buff_mgr: BuffMgr,
}

impl FightCalculateDataMgr {
    pub fn new(fight: Arc<Fight>) -> Self {
        Self {
            fight: fight.clone(),
            entity_mgr: FightEntityDataMgr::new(fight.clone()),
            buff_mgr: BuffMgr::new(),
        }
    }

    pub fn play_step_data(
        &mut self,
        step: &FightStep,
        fight: &mut Fight,
        bloodtithe: &mut BloodtitheState,
        buff_mgr: &mut BuffMgr,
    ) -> Result<(), String> {
        for effect in &step.act_effect {
            self.play_act_effect_data(effect, fight, bloodtithe, buff_mgr)?;
        }
        Ok(())
    }

    pub fn play_step_data_list(
        &mut self,
        steps: &[FightStep],
        fight: &mut Fight,
        bloodtithe: &mut BloodtitheState,
        buff_mgr: &mut BuffMgr,
    ) -> Result<(), String> {
        for step in steps {
            self.play_step_data(step, fight, bloodtithe, buff_mgr)?;
        }
        Ok(())
    }

    pub fn play_act_effect_data(
        &mut self,
        effect: &ActEffect,
        fight: &mut Fight,
        bloodtithe: &mut BloodtitheState,
        buff_mgr: &mut BuffMgr,
    ) -> Result<(), String> {
        if let Some(ref nested) = effect.fight_step {
            return self.play_step_data(nested, fight, bloodtithe, buff_mgr);
        }

        let effect_type = EffectType::from(effect.effect_type.unwrap_or(0));

        match effect_type {
            // Just ignore
            EffectType::None | EffectType::FightStep | EffectType::MasterHalo => Ok(()),

            EffectType::Damage
            | EffectType::Crit
            | EffectType::DamageExtra
            | EffectType::OriginDamage
            | EffectType::OriginCrit
            | EffectType::DamageFromAbsorb
            | EffectType::DamageFromLostHp
            | EffectType::EnchantBurnDamage
            | EffectType::DamageShareHp
            | EffectType::DeadlyPoisonOriginDamage
            | EffectType::DeadlyPoisonOriginCrit
            | EffectType::AdditionalDamage
            | EffectType::AdditionalDamageCrit
            | EffectType::ShareHurt
            | EffectType::EnchantDepresseDamage => {
                self.play_effect_damage(effect, fight, bloodtithe)
            }

            EffectType::Heal
            | EffectType::Bloodlust
            | EffectType::InjuryBankHeal
            | EffectType::SubHeroLifeChange => self.play_effect_heal(effect, fight),

            EffectType::BuffAdd => self.play_effect_add_buff(effect),

            EffectType::Dead => self.play_effect_death(effect, fight),
            EffectType::Kill => self.play_effect_kill(effect, fight),

            EffectType::Shield => self.play_effect_shield(effect, fight),

            EffectType::AverageLife => self.play_effect_set_hp(effect, fight),
            EffectType::MaxHpChange => self.play_effect_set_max_hp(effect, fight),
            EffectType::CurrentHpChange => self.play_effect_set_current_hp(effect, fight),

            EffectType::AddExPoint | EffectType::ExPointChange => {
                self.play_effect_add_ex_point(effect, fight)
            }

            EffectType::BloodPoolMaxCreate => self.play_effect_bloodtithe_enable(effect),
            EffectType::BloodPoolMaxChange => self.play_effect_bloodtithe_max(effect),
            EffectType::BloodPoolValueChange => self.play_effect_bloodtithe_value(effect),

            EffectType::FightHurtDetail => self.play_effect_fight_hurt_detail(effect, fight),

            // TODO
            EffectType::EnterFightDeal
            | EffectType::Attr
            | EffectType::TeammateInjuryCount
            | EffectType::ExPointOverflowBank
            | EffectType::Cure
            | EffectType::CardsPush
            | EffectType::CardDeckNum => Ok(()),

            other => {
                tracing::warn!("Unhandled effect type: {:?}", other);
                Ok(())
            }
        }
    }

    fn play_effect_damage(
        &mut self,
        effect: &ActEffect,
        fight: &mut Fight,
        bloodtithe: &mut BloodtitheState,
    ) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;
        let damage = effect.effect_num.ok_or("No damage amount")?;

        let location = self
            .entity_mgr
            .get_location(target_id)
            .ok_or_else(|| format!("Entity {} not found", target_id))?;

        let entity = get_entity_mut_by_location(fight, location)
            .ok_or_else(|| format!("Failed to get entity {} mutably", target_id))?;

        let current_hp = entity.current_hp.unwrap_or(0);
        entity.current_hp = Some((current_hp - damage).max(0));

        if let Some(team_type) = entity.team_type
            && damage > 0
        {
            bloodtithe.on_hp_lost(target_id, team_type, damage);
        }

        tracing::trace!("Damage applied: target={}, damage={}", target_id, damage);
        Ok(())
    }

    fn play_effect_heal(&mut self, effect: &ActEffect, fight: &mut Fight) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;
        let heal = effect.effect_num.ok_or("No heal amount")?;

        let location = self
            .entity_mgr
            .get_location(target_id)
            .ok_or_else(|| format!("Entity {} not found", target_id))?;

        let entity = get_entity_mut_by_location(fight, location)
            .ok_or_else(|| format!("Failed to get entity {} mutably", target_id))?;

        let current_hp = entity.current_hp.unwrap_or(0);
        let max_hp = entity
            .attr
            .as_ref()
            .and_then(|a| a.hp)
            .unwrap_or(current_hp);
        entity.current_hp = Some((current_hp + heal).min(max_hp));

        tracing::trace!("Heal applied: target={}, heal={}", target_id, heal);
        Ok(())
    }

    fn play_effect_add_buff(&mut self, effect: &ActEffect) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;
        let buff_id = effect.effect_num.ok_or("No buff ID")?;

        let from_uid = effect.buff.as_ref().and_then(|b| b.from_uid).unwrap_or(0);

        self.buff_mgr.add_buff(target_id, buff_id, from_uid);

        Ok(())
    }

    fn play_effect_death(&mut self, effect: &ActEffect, fight: &mut Fight) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;

        let location = self
            .entity_mgr
            .get_location(target_id)
            .ok_or_else(|| format!("Entity {} not found", target_id))?;

        let entity = get_entity_mut_by_location(fight, location)
            .ok_or_else(|| format!("Failed to get entity {} mutably", target_id))?;

        entity.current_hp = Some(0);
        self.buff_mgr.clear_dead(target_id);

        tracing::trace!("Entity died: target={}", target_id);
        Ok(())
    }

    fn play_effect_kill(&mut self, effect: &ActEffect, fight: &mut Fight) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;

        let location = self
            .entity_mgr
            .get_location(target_id)
            .ok_or_else(|| format!("Entity {} not found", target_id))?;

        let entity = get_entity_mut_by_location(fight, location)
            .ok_or_else(|| format!("Failed to get entity {} mutably", target_id))?;

        entity.current_hp = Some(0);
        self.buff_mgr.clear_dead(target_id);

        tracing::trace!("Entity killed: target={}", target_id);
        Ok(())
    }

    fn play_effect_shield(&mut self, effect: &ActEffect, _fight: &mut Fight) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;
        let shield = effect.effect_num.ok_or("No shield amount")?;

        tracing::trace!("Shield applied: target={}, shield={}", target_id, shield);
        // TODO: Add shield to entity
        Ok(())
    }

    fn play_effect_set_hp(&mut self, effect: &ActEffect, fight: &mut Fight) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;
        let hp = effect.effect_num.ok_or("No HP amount")?;

        let location = self
            .entity_mgr
            .get_location(target_id)
            .ok_or_else(|| format!("Entity {} not found", target_id))?;

        let entity = get_entity_mut_by_location(fight, location)
            .ok_or_else(|| format!("Failed to get entity {} mutably", target_id))?;

        entity.current_hp = Some(hp);
        tracing::trace!("HP set: target={}, hp={}", target_id, hp);
        Ok(())
    }

    fn play_effect_set_max_hp(
        &mut self,
        effect: &ActEffect,
        fight: &mut Fight,
    ) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;
        let max_hp = effect.effect_num.ok_or("No max HP amount")?;

        let location = self
            .entity_mgr
            .get_location(target_id)
            .ok_or_else(|| format!("Entity {} not found", target_id))?;

        let entity = get_entity_mut_by_location(fight, location)
            .ok_or_else(|| format!("Failed to get entity {} mutably", target_id))?;

        if let Some(attr) = entity.attr.as_mut() {
            attr.hp = Some(max_hp);
        }

        if let Some(base) = entity.base_attr.as_mut() {
            base.hp = Some(max_hp);
        }

        tracing::trace!("Max HP set: target={}, max_hp={}", target_id, max_hp);
        Ok(())
    }

    fn play_effect_set_current_hp(
        &mut self,
        effect: &ActEffect,
        fight: &mut Fight,
    ) -> Result<(), String> {
        self.play_effect_set_hp(effect, fight)
    }

    fn play_effect_add_ex_point(
        &mut self,
        effect: &ActEffect,
        _fight: &mut Fight,
    ) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;
        let ex_point = effect.effect_num.unwrap_or(0);

        tracing::trace!("EX point added: target={}, amount={}", target_id, ex_point);
        // TODO: Add EX points to entity
        Ok(())
    }

    fn play_effect_fight_hurt_detail(
        &mut self,
        effect: &ActEffect,
        _fight: &mut Fight,
    ) -> Result<(), String> {
        let target_id = effect.target_id.ok_or("No target ID")?;
        let hurt = effect.effect_num.unwrap_or(0);

        tracing::trace!("Hurt detail: target={}, amount={}", target_id, hurt);

        Ok(())
    }

    fn play_effect_bloodtithe_enable(&mut self, effect: &ActEffect) -> Result<(), String> {
        let team_type = effect.team_type.unwrap_or(1);
        tracing::trace!("Bloodtithe enabled: team={}", team_type);
        Ok(())
    }

    fn play_effect_bloodtithe_max(&mut self, effect: &ActEffect) -> Result<(), String> {
        let team_type = effect.team_type.unwrap_or(1);
        let max = effect.effect_num1.unwrap_or(0);
        tracing::trace!("Bloodtithe max: team={}, max={}", team_type, max);
        Ok(())
    }

    fn play_effect_bloodtithe_value(&mut self, effect: &ActEffect) -> Result<(), String> {
        let target_id = effect.target_id.unwrap_or(0);
        let value = effect.effect_num.unwrap_or(0);
        tracing::trace!("Bloodtithe value: target={}, value={}", target_id, value);
        Ok(())
    }

    pub fn update_fight(&mut self, fight: Arc<Fight>) {
        self.fight = fight.clone();
        self.entity_mgr.update_fight(fight);
    }
}

impl FightCalculateDataMgr {
    pub fn build_ex_point_info(&mut self, fight: &Fight) -> Vec<FightExPointInfo> {
        let mut info = Vec::new();

        if let Some(ref attacker) = fight.attacker {
            for entity in &attacker.entitys {
                info.push(FightExPointInfo {
                    uid: entity.uid,
                    ex_point: entity.ex_point,
                    power_infos: vec![],
                    current_hp: entity.current_hp,
                    ex_point_type: if entity.model_id == Some(3120) {
                        Some(1)
                    } else {
                        Some(0)
                    },
                });
            }

            for entity in &attacker.sub_entitys {
                info.push(FightExPointInfo {
                    uid: entity.uid,
                    ex_point: entity.ex_point,
                    power_infos: vec![],
                    current_hp: entity.current_hp,
                    ex_point_type: Some(0),
                });
            }
        }

        if let Some(ref defender) = fight.defender {
            for entity in &defender.entitys {
                info.push(FightExPointInfo {
                    uid: entity.uid,
                    ex_point: entity.ex_point,
                    power_infos: vec![],
                    current_hp: entity.current_hp,
                    ex_point_type: Some(0),
                });
            }
        }

        info
    }

    pub fn build_hero_sp_attributes(&mut self, fight: &Fight) -> Vec<FightHeroSpAttributeInfo> {
        let mut attrs = vec![];

        if let Some(ref defender) = fight.defender {
            for entity in &defender.entitys {
                attrs.push(FightHeroSpAttributeInfo {
                    uid: entity.uid,
                    attribute: Some(HeroSpAttribute {
                        revive: Some(0),
                        heal: Some(0),
                        absorb: Some(0),
                        defense_ignore: Some(0),
                        clutch: Some(0),
                        final_add_dmg: Some(0),
                        final_drop_dmg: Some(0),
                        normal_skill_rate: Some(0),
                        play_add_rate: Some(0),
                        play_drop_rate: Some(0),
                        dizzy_resistances: Some(0),
                        sleep_resistances: Some(0),
                        petrified_resistances: Some(0),
                        frozen_resistances: Some(0),
                        disarm_resistances: Some(0),
                        forbid_resistances: Some(0),
                        seal_resistances: Some(0),
                        cant_get_exskill_resistances: Some(0),
                        del_ex_point_resistances: Some(0),
                        stress_up_resistances: Some(0),
                        control_resilience: Some(0),
                        del_ex_point_resilience: Some(0),
                        stress_up_resilience: Some(0),
                        charm_resistances: Some(0),
                        rebound_dmg: Some(0),
                        extra_dmg: Some(0),
                        reuse_dmg: Some(0),
                        big_skill_rate: Some(0),
                        clutch_dmg: Some(0),
                    }),
                });
            }
        }

        attrs
    }

    pub fn build_player_skills(&mut self) -> Vec<PlayerSkillInfo> {
        vec![
            PlayerSkillInfo {
                skill_id: Some(30010201),
                cd: Some(0),
                need_power: Some(40),
                r#type: Some(0),
            },
            PlayerSkillInfo {
                skill_id: Some(30010202),
                cd: Some(0),
                need_power: Some(25),
                r#type: Some(0),
            },
        ]
    }
}

impl FightCalculateDataMgr {
    pub fn on_round_end(&mut self) {
        self.buff_mgr.on_round_end();
    }
}
