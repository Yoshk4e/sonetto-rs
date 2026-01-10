use std::sync::Arc;

use chrono::Datelike;
use database::models::game::heros::UserHeroModel;
use rand::{seq::SliceRandom, thread_rng};
use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::{
    error::AppError,
    state::{ConnectionContext, get_rewards, parse_item},
    utils::{
        inventory::{add_currencies, add_items},
        push::{self, send_red_dot_push},
    },
};

pub fn process_item_use(
    material_id: u32,
    quantity: i32,
    target_id: Option<u64>,
) -> (Vec<(u32, i32)>, Vec<(i32, i32)>) {
    let is_selector = material_id >= 481000 && material_id <= 481020;

    let is_hero_selector = material_id == 481022;
    if is_hero_selector && target_id.is_some() {
        return (vec![], vec![]);
    }

    if is_selector && target_id.is_some() {
        let (items, currencies) = get_rewards(material_id);
        let target_idx = target_id.unwrap() as usize;

        if let Some(item) = items.get(target_idx) {
            (vec![(item.0, item.1 * quantity)], vec![])
        } else if let Some(currency) = currencies.get(target_idx) {
            (vec![], vec![(currency.0, currency.1 * quantity)])
        } else {
            tracing::warn!(
                "Invalid target_id {} for selector item {}",
                target_idx,
                material_id
            );
            (vec![], vec![])
        }
    } else if target_id.unwrap_or(0) > 0 {
        let target_id_val = target_id.unwrap();
        (vec![(target_id_val as u32, quantity)], vec![])
    } else {
        let game_data = data::exceldb::get();
        let item_cfg = game_data.item.get(material_id as i32);

        if let Some(cfg) = item_cfg {
            if let Some((items, currencies)) = parse_item(&cfg.effect) {
                let final_items: Vec<(u32, i32)> = items
                    .iter()
                    .map(|(id, amt)| (*id, amt * quantity))
                    .collect();
                let final_currencies: Vec<(i32, i32)> = currencies
                    .iter()
                    .map(|(id, amt)| (*id, amt * quantity))
                    .collect();
                (final_items, final_currencies)
            } else {
                let (items, currencies) = get_rewards(material_id);
                let final_items = if items.len() > 1 {
                    let mut rng = thread_rng();
                    let mut selected = Vec::new();
                    for _ in 0..quantity {
                        if let Some(random_item) = items.choose(&mut rng) {
                            selected.push(*random_item);
                        }
                    }
                    selected
                } else {
                    items
                        .iter()
                        .map(|(id, amt)| (*id, amt * quantity))
                        .collect()
                };
                let final_currencies: Vec<(i32, i32)> = currencies
                    .iter()
                    .map(|(id, amt)| (*id, amt * quantity))
                    .collect();
                (final_items, final_currencies)
            }
        } else {
            tracing::warn!("Item {} not found in game data", material_id);
            (vec![], vec![])
        }
    }
}

pub async fn apply_insight_item(
    pool: &SqlitePool,
    player_id: i64,
    uid: i64,
    hero_id: i32,
) -> Result<i32, AppError> {
    let (item_id, quantity): (i32, i32) =
        sqlx::query_as("SELECT item_id, quantity FROM insight_items WHERE uid = ? AND user_id = ?")
            .bind(uid)
            .bind(player_id)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::InvalidRequest)?;

    if quantity <= 0 {
        tracing::warn!(
            "User {} tried to use insight item with 0 quantity (uid: {})",
            player_id,
            uid
        );
        return Ok(item_id);
    }

    let game_data = data::exceldb::get();
    let insight_data = game_data
        .insight_item
        .iter()
        .find(|i| i.id == item_id)
        .ok_or(AppError::InvalidRequest)?;

    let hero = UserHeroModel::new(player_id, pool.clone());

    let hero_data = hero.get_hero(hero_id).await?;

    let target_rank = insight_data.hero_rank + 1;
    let target_level = insight_data
        .effect
        .split('#')
        .nth(1)
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    sqlx::query("UPDATE heroes SET rank = ?, level = ? WHERE user_id = ? AND hero_id = ?")
        .bind(target_rank)
        .bind(target_level)
        .bind(player_id)
        .bind(hero_id)
        .execute(pool)
        .await?;

    if target_rank >= 3 {
        unlock_insight_skin(pool, player_id, hero_id, hero_data.record.uid).await?;
    }

    sqlx::query(
        "UPDATE insight_items
         SET quantity = quantity - 1
         WHERE uid = ? AND user_id = ?",
    )
    .bind(uid)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(item_id)
}

async fn unlock_insight_skin(
    pool: &SqlitePool,
    player_id: i64,
    hero_id: i32,
    hero_uid: i64,
) -> Result<(), AppError> {
    let game_data = data::exceldb::get();
    let Some(skin) = game_data
        .skin
        .iter()
        .find(|s| s.character_id == hero_id && s.id % 100 == 2 && s.gain_approach == 1)
    else {
        return Ok(());
    };

    let has_skin: Option<i32> =
        sqlx::query_scalar("SELECT 1 FROM hero_all_skins WHERE user_id = ? AND skin_id = ?")
            .bind(player_id)
            .bind(skin.id)
            .fetch_optional(pool)
            .await?;

    if has_skin.is_some() {
        return Ok(());
    }

    sqlx::query("INSERT INTO hero_all_skins (user_id, skin_id) VALUES (?, ?)")
        .bind(player_id)
        .bind(skin.id)
        .execute(pool)
        .await?;

    sqlx::query("INSERT INTO hero_skins (hero_uid, skin, expire_sec) VALUES (?, ?, 0)")
        .bind(hero_uid)
        .bind(skin.id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE heroes SET skin = ? WHERE uid = ? AND user_id = ?")
        .bind(skin.id)
        .bind(hero_uid)
        .bind(player_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn can_claim_month_card(
    ctx: Arc<Mutex<ConnectionContext>>,
    player_id: i64,
) -> Result<(), AppError> {
    let pool = {
        let ctx_guard = ctx.lock().await;
        ctx_guard.state.db.clone()
    };

    let current_time = common::time::ServerTime::now_ms();
    let server_day = common::time::ServerTime::server_day(current_time);
    let adjusted = common::time::ServerTime::adjusted_datetime(current_time);
    let day_of_month = adjusted.day() as i32;

    let active_cards: Vec<(i32, i64)> = sqlx::query_as(
        "SELECT card_id, end_time
         FROM user_month_card_history
         WHERE user_id = ? AND end_time > ?
         ORDER BY card_id",
    )
    .bind(player_id)
    .bind(current_time / 1000)
    .fetch_all(&pool)
    .await?;

    if active_cards.is_empty() {
        return Ok(());
    }

    let already_claimed: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM user_month_card_days
         WHERE user_id = ? AND server_day = ?",
    )
    .bind(player_id)
    .bind(server_day)
    .fetch_optional(&pool)
    .await?;

    if already_claimed.is_some() {
        tracing::info!("User {} already claimed month card today", player_id);
        return Ok(());
    }

    let logged_in_today = {
        let ctx_guard = ctx.lock().await;
        matches!(ctx_guard.player_state.as_ref(),
            Some(s) if s.last_sign_in_day == server_day)
    };

    if !logged_in_today {
        tracing::info!(
            "User {} not logged in today, skipping month card claim",
            player_id
        );
        return Ok(());
    }

    let inserted = sqlx::query(
        "INSERT OR IGNORE INTO user_month_card_days (user_id, server_day, day_of_month)
         VALUES (?, ?, ?)",
    )
    .bind(player_id)
    .bind(server_day)
    .bind(day_of_month)
    .execute(&pool)
    .await?
    .rows_affected();

    if inserted != 1 {
        return Ok(());
    }

    tracing::info!(
        "Auto-claiming month card daily bonus (user_id={}, server_day={}, day={})",
        player_id,
        server_day,
        day_of_month
    );

    let game_data = data::exceldb::get();
    let mut reward_str = String::new();

    for (card_id, _) in &active_cards {
        if let Some(card) = game_data.month_card.iter().find(|c| c.id == *card_id) {
            if !reward_str.is_empty() {
                reward_str.push('|');
            }
            reward_str.push_str(&card.daily_bonus);
        }
    }

    let (items, currencies, _, _, power_items, _) = crate::state::parse_store_product(&reward_str);

    let mut material_changes = Vec::new();

    if !items.is_empty() {
        let ids = add_items(&pool, player_id, &items).await?;
        for (id, amt) in items {
            material_changes.push((1, id, amt));
        }
        push::send_item_change_push(ctx.clone(), player_id, ids, vec![], vec![]).await?;
    }

    if !currencies.is_empty() {
        let ids = add_currencies(&pool, player_id, &currencies).await?;
        for (id, amt) in currencies {
            material_changes.push((2, id as u32, amt));
        }
        push::send_currency_change_push(ctx.clone(), player_id, ids).await?;
    }

    if !power_items.is_empty() {
        database::db::game::items::add_power_items(
            &pool,
            player_id,
            &power_items
                .iter()
                .map(|(id, count)| (*id as i32, *count))
                .collect::<Vec<_>>(),
        )
        .await?;

        for (id, amt) in power_items {
            material_changes.push((10, id, amt));
        }
    }

    if !material_changes.is_empty() {
        push::send_material_change_push(ctx.clone(), material_changes, Some(10)).await?;
    }

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .update_and_save_player_state(|state| {
                state.last_daily_reward_time = Some(current_time);
                state.claim_month_card(current_time);
                state.mark_activity_pushes_sent(current_time);
            })
            .await?;
    }

    send_red_dot_push(ctx.clone(), player_id, Some(vec![1040])).await?;

    Ok(())
}
