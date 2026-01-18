use super::utils::*;
use sonettobuf::{ActEffect, Fight, FightStep, effect_type_enum::EffectType, fight_step};

pub struct FightStepBuilder {
    act_type: fight_step::ActType,
    from_id: i64,
    to_id: i64,
    act_id: i32,
    effects: Vec<ActEffect>,
}

#[allow(dead_code)]
impl FightStepBuilder {
    pub fn new_effect() -> Self {
        Self {
            act_type: fight_step::ActType::Effect,
            from_id: 0,
            to_id: 0,
            act_id: 0,
            effects: Vec::new(),
        }
    }

    pub fn new_skill(from_id: i64, to_id: i64, skill_id: i32) -> Self {
        Self {
            act_type: fight_step::ActType::Skill,
            from_id,
            to_id,
            act_id: skill_id,
            effects: Vec::new(),
        }
    }

    pub fn add_effect(mut self, effect: ActEffect) -> Self {
        self.effects.push(effect);
        self
    }

    pub fn add_effects(mut self, effects: Vec<ActEffect>) -> Self {
        self.effects.extend(effects);
        self
    }

    pub fn add_battle_container(
        mut self,
        from_uid: i64,
        skill_id: i32,
        effects: Vec<ActEffect>,
    ) -> Self {
        let nested_step = FightStep {
            act_type: Some(fight_step::ActType::Skill.into()),
            from_id: Some(from_uid),
            to_id: Some(from_uid),
            act_id: Some(skill_id),
            act_effect: effects,
            card_index: Some(0),
            support_hero_id: Some(0),
            fake_timeline: Some(false),
        };

        self.effects.push(ActEffect {
            effect_type: Some(EffectType::Fightstep as i32),
            target_id: Some(0),
            effect_num: Some(0),
            fight_step: Some(nested_step),
            ..Default::default()
        });
        self
    }

    pub fn add_effect_container(
        mut self,
        target_uid: i64,
        skill_id: i32,
        from_uid: i64,
        effects: Vec<ActEffect>,
    ) -> Self {
        let nested_step = FightStep {
            act_type: Some(fight_step::ActType::Effect.into()),
            from_id: Some(from_uid),
            to_id: Some(target_uid),
            act_id: Some(skill_id),
            act_effect: effects,
            card_index: Some(0),
            support_hero_id: Some(0),
            fake_timeline: Some(false),
        };

        self.effects.push(ActEffect {
            effect_type: Some(EffectType::Fightstep as i32),
            target_id: Some(0),
            effect_num: Some(0),
            fight_step: Some(nested_step),
            ..Default::default()
        });
        self
    }

    pub fn add_bloodtithe_ui_sync(
        mut self,
        team: i32,
        display_uid: i64,
        cur: i32,
        max: i32,
    ) -> Self {
        self.effects.push(ActEffect {
            effect_type: Some(EffectType::Bloodpoolmaxcreate as i32),
            target_id: Some(0),
            team_type: Some(team),
            effect_num: Some(1),
            ..Default::default()
        });

        self.effects.push(ActEffect {
            effect_type: Some(EffectType::Bloodpoolmaxchange as i32),
            target_id: Some(0),
            team_type: Some(team),
            effect_num: Some(1),
            effect_num1: Some(max),
            ..Default::default()
        });

        self.effects.push(ActEffect {
            effect_type: Some(EffectType::Bloodpoolvaluechange as i32),
            target_id: Some(display_uid),
            team_type: Some(team),
            effect_num: Some(cur),
            effect_num1: Some(1),
            ..Default::default()
        });
        self
    }

    pub fn add_simple_effect(
        mut self,
        effect_type: i32,
        target_id: i64,
        team_type: Option<i32>,
        effect_num: Option<i32>,
    ) -> Self {
        self.effects.push(ActEffect {
            effect_type: Some(effect_type),
            target_id: Some(target_id),
            team_type,
            effect_num,
            ..Default::default()
        });
        self
    }

    pub fn add_nested_step(mut self, step: FightStep) -> Self {
        self.effects.push(ActEffect {
            effect_type: Some(EffectType::Fightstep as i32),
            fight_step: Some(step),
            target_id: Some(0),
            ..Default::default()
        });
        self
    }

    pub fn build_as_protected_skill(self) -> ActEffect {
        let inner_skill = self.build();

        let first_wrap = ActEffect {
            effect_type: Some(EffectType::Fightstep as i32),
            fight_step: Some(inner_skill),
            target_id: Some(0),
            effect_num: Some(0),
            ..Default::default()
        };

        let middle_step = FightStep {
            act_type: Some(fight_step::ActType::Effect.into()),
            from_id: Some(0),
            to_id: Some(0),
            act_id: Some(0),
            act_effect: vec![first_wrap],
            card_index: Some(0),
            support_hero_id: Some(0),
            fake_timeline: Some(false),
        };

        ActEffect {
            effect_type: Some(EffectType::Fightstep as i32),
            fight_step: Some(middle_step),
            target_id: Some(0),
            effect_num: Some(0),
            ..Default::default()
        }
    }

    pub fn build_as_act_effect(self) -> ActEffect {
        ActEffect {
            effect_type: Some(EffectType::Fightstep as i32),
            target_id: Some(0),
            effect_num: Some(0),
            fight_step: Some(self.build()),
            ..Default::default()
        }
    }

    pub fn build(self) -> FightStep {
        FightStep {
            act_type: Some(self.act_type.into()),
            from_id: Some(self.from_id),
            to_id: Some(self.to_id),
            act_id: Some(self.act_id),
            act_effect: self.effects,
            card_index: Some(0),
            support_hero_id: Some(0),
            fake_timeline: Some(false),
        }
    }
}

#[allow(dead_code)]
impl FightStepBuilder {
    pub fn create_team_buff_step(
        from_uid: i64,
        skill_id: i32,
        fight: &Fight,
        team_type: i32,
        buff_id: i32,
    ) -> FightStep {
        let team_uids = get_team_uids(fight, team_type);
        let mut inner_effects = Vec::new();

        for target_uid in team_uids {
            inner_effects.push(buff_add(target_uid, from_uid, buff_id));
        }

        FightStepBuilder::new_effect()
            .add_effect_container(0, skill_id, from_uid, inner_effects)
            .build()
    }
}
