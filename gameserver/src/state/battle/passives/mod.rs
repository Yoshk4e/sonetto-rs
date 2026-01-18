mod activity;
mod equipment;
mod hero_3088;
mod hero_3120;
mod hero_3125;

use anyhow::Result;
use sonettobuf::{ActEffect, Fight, FightEntityInfo};

use crate::state::battle::mechanics::bloodtithe::BloodtitheState;

pub fn build_battle_start_passives(
    entity: &FightEntityInfo,
    fight: &Fight,
    bloodtithe: &mut BloodtitheState,
) -> Result<Vec<ActEffect>> {
    let model_id = entity.model_id.unwrap_or(0);

    let mut effects = match model_id {
        3088 => hero_3088::build_battle_start(entity, fight, bloodtithe)?,
        3120 => hero_3120::build_battle_start(entity, fight, bloodtithe)?,
        3125 => hero_3125::build_battle_start(entity, fight, bloodtithe)?,
        _ => vec![],
    };

    effects.extend(equipment::build_battle_start(entity, fight)?);

    Ok(effects)
}

pub fn build_post_power_passives(
    entity: &FightEntityInfo,
    fight: &Fight,
) -> Result<Vec<ActEffect>> {
    let model_id = entity.model_id.unwrap_or(0);

    match model_id {
        3088 => hero_3088::build_post_power(entity, fight),
        3120 => hero_3120::build_post_power(entity, fight),
        3125 => hero_3125::build_post_power(entity, fight),
        _ => Ok(vec![]),
    }
}

pub fn build_round_start_passives(
    entity: &FightEntityInfo,
    fight: &Fight,
    bloodtithe: &mut BloodtitheState,
) -> Result<Vec<ActEffect>> {
    let model_id = entity.model_id.unwrap_or(0);

    match model_id {
        3088 => hero_3088::build_round_start(entity, fight, bloodtithe),
        3120 => hero_3120::build_round_start(entity, fight),
        3125 => hero_3125::build_round_start(entity, fight),
        _ => Ok(vec![]),
    }
}

pub fn build_bootstrap(entity: &FightEntityInfo) -> Result<Vec<ActEffect>> {
    let mut effects = Vec::new();

    effects.extend(activity::build_battle_start(entity)?);

    Ok(effects)
}
