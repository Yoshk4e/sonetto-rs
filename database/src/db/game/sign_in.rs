use crate::models::game::sign_in::{MonthCardHistory, UserSignInInfo};
use anyhow::Result;
use chrono::Datelike;
use common::time::ServerTime;
use sqlx::SqlitePool;

/// Record sign-in for today
async fn record_sign_in_day(pool: &SqlitePool, user_id: i64, now: i64) -> Result<bool> {
    let server_day = ServerTime::server_day(now);
    let day_of_month = ServerTime::day_of_month(now) as i32;

    let already_signed: Option<i32> =
        sqlx::query_scalar("SELECT 1 FROM user_sign_in_days WHERE user_id = ? AND server_day = ?")
            .bind(user_id)
            .bind(server_day)
            .fetch_optional(pool)
            .await?;

    if already_signed.is_some() {
        return Ok(false);
    }

    sqlx::query(
        "INSERT INTO user_sign_in_days (user_id, server_day, day_of_month)
         VALUES (?, ?, ?)
         ON CONFLICT(user_id, server_day) DO NOTHING",
    )
    .bind(user_id)
    .bind(server_day)
    .bind(day_of_month)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO user_sign_in_info
            (user_id, addup_sign_in_day, open_function_time, reward_mark)
        VALUES (?, 1, ?, 0)
        ON CONFLICT(user_id)
        DO UPDATE SET addup_sign_in_day = addup_sign_in_day + 1
        "#,
    )
    .bind(user_id)
    .bind(now / 1000)
    .execute(pool)
    .await?;

    tracing::info!(
        "Sign-in recorded user_id={} day={} server_day={}",
        user_id,
        day_of_month,
        server_day
    );

    Ok(true)
}

/// Process daily login - returns (is_new_day, is_new_week, is_new_month)
pub async fn process_daily_login(pool: &SqlitePool, user_id: i64) -> Result<(bool, bool, bool)> {
    let now = ServerTime::now_ms();

    let users_rows = sqlx::query("UPDATE users SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();

    if users_rows == 0 {
        anyhow::bail!("users row missing for user_id={}", user_id);
    }

    let last_sign_in_time: Option<i64> =
        sqlx::query_scalar("SELECT last_sign_in_time FROM player_state WHERE player_id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    let is_new_day = match last_sign_in_time {
        Some(last) if last > 0 => ServerTime::is_new_day(last, now),
        _ => true,
    };

    let is_new_week = match last_sign_in_time {
        Some(last) if last > 0 => !ServerTime::is_same_week(last, now),
        _ => true,
    };

    let is_new_month = match last_sign_in_time {
        Some(last) if last > 0 => !ServerTime::is_same_month(last, now),
        _ => true,
    };

    if is_new_month {
        sqlx::query("DELETE FROM user_sign_in_days WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM user_month_card_days WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM user_sign_in_addup_bonus WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;

        reset_monthly_counters(pool, user_id).await?;
    }

    if is_new_week {
        reset_weekly_counters(pool, user_id).await?;
    }

    if is_new_day {
        record_sign_in_day(pool, user_id, now).await?;

        reset_daily_counters(pool, user_id).await?;
    }

    let ps_rows = sqlx::query(
        "UPDATE player_state
         SET last_sign_in_time = ?,
            updated_at = ?,
            initial_login_complete = 0,
            month_card_claimed = 0,
            last_month_card_claim_timestamp = NULL
         WHERE player_id = ?",
    )
    .bind(now)
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected();

    if ps_rows == 0 {
        anyhow::bail!("player_state row missing for user_id={}", user_id);
    }

    Ok((is_new_day, is_new_week, is_new_month))
}

pub async fn process_manual_sign_in(pool: &SqlitePool, user_id: i64) -> Result<bool> {
    let now = ServerTime::now_ms();

    let recorded = record_sign_in_day(pool, user_id, now).await?;

    if !recorded {
        tracing::info!("User {} already signed in today", user_id);
    }

    Ok(recorded)
}

/// Reset daily counters (call this for any daily-reset systems)
pub async fn reset_daily_counters(pool: &SqlitePool, user_id: i64) -> Result<()> {
    // Reset dungeon daily attempts
    sqlx::query(
        "UPDATE user_dungeons SET today_pass_num = 0, today_total_num = 0 WHERE user_id = ?",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    // Reset chapter type daily nums
    sqlx::query(
        "UPDATE user_chapter_type_nums SET today_pass_num = 0, today_total_num = 0 WHERE user_id = ?"
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    // Reset hero touch count
    sqlx::query(
        r#"
        INSERT INTO hero_touch_count (user_id, touch_count_left)
        VALUES (?, 5)
        ON CONFLICT(user_id) DO UPDATE SET touch_count_left = 5
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    tracing::info!("Reset daily counters for user {}", user_id);
    Ok(())
}

/// Reset weekly counters (call this for any weekly-reset systems)
pub async fn reset_weekly_counters(pool: &SqlitePool, user_id: i64) -> Result<()> {
    let game_data = data::exceldb::get();

    let weekly_store_goods: Vec<i32> = game_data
        .store_goods
        .iter()
        .filter(|g| g.refresh_time == 2)
        .map(|g| g.id)
        .collect();

    if !weekly_store_goods.is_empty() {
        for goods_id in &weekly_store_goods {
            sqlx::query(
                "UPDATE user_store_goods
                 SET buy_count = 0
                 WHERE user_id = ? AND goods_id = ?",
            )
            .bind(user_id)
            .bind(goods_id)
            .execute(pool)
            .await?;
        }

        tracing::info!(
            "Reset weekly store_goods for user {}: {} items reset",
            user_id,
            weekly_store_goods.len()
        );
    }

    let weekly_charge_goods: Vec<i32> = game_data
        .store_charge_goods
        .iter()
        .filter(|g| g.order == 40)
        .map(|g| g.id)
        .collect();

    if !weekly_charge_goods.is_empty() {
        for goods_id in &weekly_charge_goods {
            sqlx::query(
                "UPDATE user_charge_info
                 SET buy_count = 0
                 WHERE user_id = ? AND charge_id = ?",
            )
            .bind(user_id)
            .bind(goods_id)
            .execute(pool)
            .await?;
        }

        tracing::info!(
            "Reset weekly store_charge_goods for user {}: {} items reset",
            user_id,
            weekly_charge_goods.len()
        );
    }

    if weekly_store_goods.is_empty() && weekly_charge_goods.is_empty() {
        tracing::info!(
            "Reset weekly counters for user {} (no weekly items)",
            user_id
        );
    }

    Ok(())
}

/// Reset monthly counters
pub async fn reset_monthly_counters(pool: &SqlitePool, user_id: i64) -> Result<()> {
    let game_data = data::exceldb::get();

    sqlx::query(
        "UPDATE user_sign_in_info
             SET addup_sign_in_day = 0
             WHERE user_id = ?",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    let monthly_store_goods: Vec<i32> = game_data
        .store_goods
        .iter()
        .filter(|g| g.refresh_time == 3)
        .map(|g| g.id)
        .collect();

    if !monthly_store_goods.is_empty() {
        for goods_id in &monthly_store_goods {
            sqlx::query(
                "UPDATE user_store_goods
                 SET buy_count = 0
                 WHERE user_id = ? AND goods_id = ?",
            )
            .bind(user_id)
            .bind(goods_id)
            .execute(pool)
            .await?;
        }

        tracing::info!(
            "Reset monthly store_goods for user {}: {} items reset",
            user_id,
            monthly_store_goods.len()
        );
    }

    let monthly_charge_goods: Vec<i32> = game_data
        .store_charge_goods
        .iter()
        .filter(|g| {
            g.belong_store_id == 614 && matches!(g.order, 120 | 311 | 312 | 350 | 500 | 600)
        })
        .map(|g| g.id)
        .collect();

    if !monthly_charge_goods.is_empty() {
        for goods_id in &monthly_charge_goods {
            sqlx::query(
                "UPDATE user_charge_info
                 SET buy_count = 0
                 WHERE user_id = ? AND charge_id = ?",
            )
            .bind(user_id)
            .bind(goods_id)
            .execute(pool)
            .await?;
        }

        tracing::info!(
            "Reset monthly store_charge_goods for user {}: {} items reset",
            user_id,
            monthly_charge_goods.len()
        );
    }

    if monthly_store_goods.is_empty() && monthly_charge_goods.is_empty() {
        tracing::info!(
            "Reset monthly counters for user {} (no monthly items)",
            user_id
        );
    }

    Ok(())
}

pub async fn ensure_sign_in_info(pool: &SqlitePool, user_id: i64) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO user_sign_in_info (user_id, addup_sign_in_day, open_function_time, reward_mark)
        VALUES (?, 0, 0, 0)
        ON CONFLICT(user_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_sign_in_info(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<(
    UserSignInInfo,
    Vec<i32>,              // sign-in days (day_of_month)
    Vec<i32>,              // addup bonus ids
    Vec<i32>,              // month card days (day_of_month)
    Vec<MonthCardHistory>, // month card history
    Vec<i32>,              // birthday heroes
)> {
    let info =
        sqlx::query_as::<_, UserSignInInfo>("SELECT * FROM user_sign_in_info WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or(UserSignInInfo {
                user_id,
                addup_sign_in_day: 0,
                open_function_time: 0,
                reward_mark: 0,
            });

    let sign_in_days = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT day_of_month
        FROM user_sign_in_days
        WHERE user_id = ?
        ORDER BY server_day
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let addup_bonus = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT bonus_id
        FROM user_sign_in_addup_bonus
        WHERE user_id = ?
        ORDER BY bonus_id
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let month_card_days = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT day_of_month
        FROM user_month_card_days
        WHERE user_id = ?
        ORDER BY server_day
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let month_card_history = sqlx::query_as::<_, MonthCardHistory>(
        r#"
        SELECT card_id, start_time, end_time
        FROM user_month_card_history
        WHERE user_id = ?
        ORDER BY start_time
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let birthday_heroes = get_birthday_heroes_today(pool, user_id).await?;

    Ok((
        info,
        sign_in_days,
        addup_bonus,
        month_card_days,
        month_card_history,
        birthday_heroes,
    ))
}

pub async fn add_sign_in_day(
    pool: &SqlitePool,
    user_id: i64,
    server_day: i32,
    day_of_month: i32,
    now_ms: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO user_sign_in_days (user_id, server_day, day_of_month)
        VALUES (?, ?, ?)
        ON CONFLICT(user_id, server_day) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(server_day)
    .bind(day_of_month)
    .execute(pool)
    .await?;

    let now_sec = (now_ms / 1000) as i32;

    sqlx::query(
        r#"
        INSERT INTO user_sign_in_info
            (user_id, addup_sign_in_day, open_function_time, reward_mark)
        VALUES (?, 1, ?, 0)
        ON CONFLICT(user_id)
        DO UPDATE SET addup_sign_in_day = addup_sign_in_day + 1
        "#,
    )
    .bind(user_id)
    .bind(now_sec)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get heroes whose birthday is today (using server time for consistency)
pub async fn get_birthday_heroes_today(pool: &SqlitePool, user_id: i64) -> Result<Vec<i32>> {
    // Use ServerTime for consistency
    let server_now = common::time::ServerTime::server_date();
    let current_month = server_now.month();
    let current_day = server_now.day();

    let game_data = data::exceldb::get();

    // Find all heroes whose birthday is today
    let mut birthday_hero_ids = Vec::new();

    let characters: Vec<_> = game_data.character.iter().collect();

    for character in &characters.clone() {
        // Parse roleBirthday format "10/23" -> month=10, day=23
        if let Some((month_str, day_str)) = character.role_birthday.split_once('/') {
            if let (Ok(month), Ok(day)) = (month_str.parse::<u32>(), day_str.parse::<u32>()) {
                if month == current_month && day == current_day {
                    birthday_hero_ids.push(character.id);
                }
            }
        }
    }

    if birthday_hero_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Filter to only heroes the user actually owns
    let placeholders = birthday_hero_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");

    let query = format!(
        "SELECT hero_id FROM heroes WHERE user_id = ? AND hero_id IN ({})",
        placeholders
    );

    let mut query = sqlx::query_scalar(&query).bind(user_id);
    for hero_id in birthday_hero_ids {
        query = query.bind(hero_id);
    }

    let owned_birthday_heroes = query.fetch_all(pool).await?;

    Ok(owned_birthday_heroes)
}
