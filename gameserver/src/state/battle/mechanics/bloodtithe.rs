use once_cell::sync::Lazy;
use sonettobuf::{ActEffect, Fight, FightEntityInfo, effect_type_enum::EffectType};
use std::collections::HashMap;
use std::sync::Mutex;

/// How much HP loss = 1 Bloodtithe
const DAMAGE_PER_POINT: i32 = 3000;

const BASE_MAX: i32 = 24;

const PER_ALLY_BONUS: i32 = 16;

static GAINED: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BloodtitheState {
    value: HashMap<i32, i32>,       // team_type -> current
    max: HashMap<i32, i32>,         // team_type -> max
    accumulator: HashMap<i32, i32>, // team_type -> accumulated HP loss
    pub initialized: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct BloodtitheRule {
    pub hero_id: i32,
    pub required_passive: Option<i32>,
}

#[allow(dead_code)]
impl BloodtitheState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recalc_max(&mut self, team_type: i32, enabler_count: i32) {
        let max = BASE_MAX + (enabler_count * PER_ALLY_BONUS);
        self.max.insert(team_type, max);

        let cur = self.value.entry(team_type).or_insert(0);
        if *cur > max {
            *cur = max;
        }

        tracing::info!(
            "Bloodtithe max recalculated: team={} enablers={} max={}",
            team_type,
            enabler_count,
            max
        );
    }

    pub fn on_hp_lost(&mut self, uid: i64, team_type: i32, hp_lost: i32) -> Option<i32> {
        let acc = self.accumulator.entry(team_type).or_insert(0);
        *acc += hp_lost;

        let max = *self.max.get(&team_type).unwrap_or(&BASE_MAX);
        let value = self.value.entry(team_type).or_insert(0);

        let mut gained = 0;
        set_gain(0);

        while *acc >= DAMAGE_PER_POINT && *value < max {
            *acc -= DAMAGE_PER_POINT;
            *value += 1;
            gained += 1;
            set_gain(gained);
            bloodtithe_add_to_pool(uid, gained);
        }

        if gained > 0 {
            tracing::info!(
                "Bloodtithe gained: team={} +{} => {}/{} (acc {}/{})",
                team_type,
                gained,
                *value,
                max,
                *acc,
                DAMAGE_PER_POINT
            );
            Some(*value)
        } else {
            None
        }
    }

    pub fn get_value(&self, team_type: i32) -> i32 {
        *self.value.get(&team_type).unwrap_or(&0)
    }

    pub fn get_max(&self, team_type: i32) -> i32 {
        *self.max.get(&team_type).unwrap_or(&BASE_MAX)
    }

    pub fn get_acc(&self, team_type: i32) -> i32 {
        *self.accumulator.get(&team_type).unwrap_or(&0)
    }

    pub fn set_value(&mut self, team_type: i32, value: i32) {
        let max = self.get_max(team_type);
        let capped_value = value.min(max);
        self.value.insert(team_type, capped_value);

        tracing::debug!(
            "Bloodtithe set_value: team={} value={} (capped at {})",
            team_type,
            value,
            capped_value
        );
    }

    pub fn add_initial_gain(&mut self, team_type: i32, amount: i32) {
        let current = self.get_value(team_type);
        let new_value = current + amount;
        self.set_value(team_type, new_value);

        set_gain(amount);

        tracing::info!(
            "Bloodtithe initial gain: team={} +{} => {}/{}",
            team_type,
            amount,
            self.get_value(team_type),
            self.get_max(team_type)
        );
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.max.clear();
        self.accumulator.clear();
    }

    pub fn reset_temporary_state(&mut self) {
        self.accumulator.clear();
        set_gain(0);
    }
}

pub fn bloodtithe_add_to_pool(target_uid: i64, value: i32) -> ActEffect {
    ActEffect {
        effect_type: Some(EffectType::Bloodpoolvaluechange as i32), // 335
        target_id: Some(target_uid),
        effect_num: Some(value),
        effect_num1: Some(1),
        ..Default::default()
    }
}

pub const BLOODTITHE_RULES: &[BloodtitheRule] = &[
    BloodtitheRule {
        hero_id: 3120,
        required_passive: Some(31200146),
    },
    BloodtitheRule {
        hero_id: 3088,
        required_passive: None,
    },
    BloodtitheRule {
        hero_id: 3125,
        required_passive: None,
    },
    BloodtitheRule {
        hero_id: 3126,
        required_passive: None,
    },
];

pub fn entity_enables_bloodtithe(entity: &FightEntityInfo) -> bool {
    if entity.position.unwrap_or(-1) <= 0 {
        return false;
    }

    if entity.current_hp.unwrap_or(0) <= 0 {
        return false;
    }

    let Some(model) = entity.model_id else {
        return false;
    };

    BLOODTITHE_RULES.iter().any(|rule| {
        model == rule.hero_id
            && match rule.required_passive {
                None => true,
                Some(p) => entity.passive_skill.contains(&p),
            }
    })
}

pub fn count_team_enablers(fight: &Fight, team_type: i32) -> i32 {
    fight
        .attacker
        .as_ref()
        .map(|a| {
            a.entitys
                .iter()
                .filter(|e| e.team_type == Some(team_type))
                .filter(|e| entity_enables_bloodtithe(e))
                .count() as i32
        })
        .unwrap_or(0)
}

pub fn fight_enables_bloodtithe(fight: &Fight) -> bool {
    fight
        .attacker
        .as_ref()
        .map(|a| a.entitys.iter().any(entity_enables_bloodtithe))
        .unwrap_or(false)
}

pub fn set_gain(value: i32) {
    *GAINED.lock().unwrap() = value;
}
