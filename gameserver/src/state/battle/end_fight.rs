use crate::error::AppError;
use crate::state::ConnectionContext;
use anyhow::Result;
use sonettobuf::{
    CmdId, EndFightPush, FightGroup, FightRecord, FightStatistics, UseCardStatistics,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct BattleStats {
    pub hero_uid: i64,
    pub harm: i64,                     // Damage dealt
    pub hurt: i64,                     // Damage taken
    pub heal: i64,                     // Healing done
    pub cards_used: HashMap<i32, i32>, // skill_id -> use_count
    pub buffs_received: Vec<i32>,      // buff_ids
}

pub async fn send_end_fight_push(
    ctx: Arc<Mutex<ConnectionContext>>,
    fight_id: i64,
    fight_result: i32, // 1 = win, 2 = lose
    fight_group: FightGroup,
    attacker_stats: Vec<BattleStats>,
    defender_stats: Vec<BattleStats>,
    is_record: bool,
) -> Result<(), AppError> {
    let fight_time = chrono::Utc::now().timestamp_millis();

    // Build attacker statistics
    let attack_statistics = attacker_stats
        .into_iter()
        .map(build_fight_statistics)
        .collect();

    // Build defender statistics
    let defense_statistics = defender_stats
        .into_iter()
        .map(build_fight_statistics)
        .collect();

    let record = FightRecord {
        fight_id: Some(fight_id),
        fight_name: Some(String::new()),
        fight_time: Some(fight_time),
        fight_result: Some(fight_result),
        attack_statistics,
        defense_statistics,
    };

    let push = EndFightPush {
        record: Some(record),
        fight_group_a: Some(fight_group),
        is_record: Some(is_record), // Use the parameter
    };

    let mut conn = ctx.lock().await;
    conn.notify(CmdId::FightEndFightPushCmd, push).await?;

    Ok(())
}

fn build_fight_statistics(stats: BattleStats) -> FightStatistics {
    // Convert cards_used HashMap to UseCardStatistics
    let cards: Vec<UseCardStatistics> = stats
        .cards_used
        .into_iter()
        .map(|(skill_id, use_count)| UseCardStatistics {
            skill_id: Some(skill_id),
            use_count: Some(use_count),
        })
        .collect();

    FightStatistics {
        hero_uid: Some(stats.hero_uid),
        harm: Some(stats.harm),
        hurt: Some(stats.hurt),
        heal: Some(stats.heal),
        cards,
        get_buffs: stats.buffs_received,
    }
}

// Helper to collect stats from simulator
#[allow(dead_code)]
pub fn collect_battle_stats(
    fight_group: &FightGroup,
    // TODO: Pass in actual battle state from simulator
) -> Vec<BattleStats> {
    // For now, return empty stats for each hero
    fight_group
        .hero_list
        .iter()
        .map(|&hero_uid| BattleStats {
            hero_uid,
            harm: 0,
            hurt: 0,
            heal: 0,
            cards_used: HashMap::new(),
            buffs_received: vec![],
        })
        .collect()
}
