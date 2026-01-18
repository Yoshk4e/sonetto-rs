mod auto;
mod cards;
mod passives;

pub mod effects;
pub mod end_fight;
pub mod entity_builder;
pub mod fight_builder;
pub mod manager;
pub mod mechanics;
pub mod rewards;
pub mod round;
pub mod round_builder;
pub mod simulator;
pub mod skill_executor;
pub mod step_builder;
pub mod utils;

use anyhow::Result;
use sonettobuf::CardInfo;
use sonettobuf::Fight;
use sonettobuf::FightRound;
use sqlx::SqlitePool;

use std::sync::atomic::AtomicI64;

pub static BUFF_UID_COUNTER: AtomicI64 = AtomicI64::new(2);

pub use auto::generate_auto_opers;

pub use cards::{default_max_ap, generate_ai_initial_deck, generate_initial_deck};

use crate::state::battle::manager::fight_data_mgr::FightDataMgr;

#[allow(dead_code)]
pub struct BattleContext {
    pub player_id: i64,
    pub chapter_id: i32,
    pub episode_id: i32,
    pub battle_id: i32,
    pub max_ap: i32,
}

pub async fn create_battle(
    pool: &SqlitePool,
    ctx: BattleContext,
    fight_group: &sonettobuf::FightGroup,
    player_deck: Vec<sonettobuf::CardInfo>,
) -> Result<(Fight, FightRound, FightDataMgr, Vec<CardInfo>)> {
    let fight = fight_builder::build_fight(pool, &ctx, fight_group).await?;

    let seed = (ctx.player_id as u64) ^ (ctx.episode_id as u64) ^ 0xA11C;
    let ai_deck = generate_ai_initial_deck(&fight, seed).await;

    let (initial_round, modified_fight, fight_data_mgr) =
        round_builder::build_initial_round(fight, player_deck, ai_deck.clone()).await?;

    Ok((modified_fight, initial_round, fight_data_mgr, ai_deck))
}
