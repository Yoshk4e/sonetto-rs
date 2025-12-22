// src/battle/mod.rs

mod cards;
pub mod end_fight;
pub mod entity_builder;
pub mod fight_builder;
pub mod rewards;
pub mod round_builder;
pub mod simulator;
pub mod step_builder;

use anyhow::Result;
use sonettobuf::StartDungeonReply;
use sqlx::SqlitePool;

pub use cards::default_max_ap;
pub use cards::generate_initial_deck;

#[allow(dead_code)]
pub struct BattleContext {
    pub player_id: i64,
    pub chapter_id: i32,
    pub episode_id: i32,
    pub battle_id: i32,
}

pub async fn create_battle(
    pool: &SqlitePool,
    ctx: BattleContext,
    fight_group: &sonettobuf::FightGroup,
    card_deck: Vec<sonettobuf::CardInfo>,
) -> Result<StartDungeonReply> {
    let fight = fight_builder::build_fight(pool, &ctx, fight_group).await?;

    let round = round_builder::build_initial_round(&fight, card_deck).await?;

    Ok(StartDungeonReply {
        fight: Some(fight),
        round: Some(round),
    })
}
