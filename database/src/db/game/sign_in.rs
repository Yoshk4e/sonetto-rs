use crate::models::game::sign_in::{MonthCardHistory, UserSignInInfo};
use anyhow::Result;
use chrono::Datelike;
use common::time::ServerTime;
use sqlx::SqlitePool;

/// Process daily login - returns (is_new_day, is_new_week, is_new_month)
pub async fn process_daily_login(pool: &SqlitePool, user_id: i64) -> Result<(bool, bool, bool)> {
    let now = common::time::ServerTime::now_ms();

    // Load last sign-in timestamp (NOT last_sign_in_day)
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
        _ => false,
    };

    let is_new_month = match last_sign_in_time {
        Some(last) if last > 0 => !ServerTime::is_same_month(last, now),
        _ => false,
    };

    /* ===============================
     * STEP 1: Month reset
     * =============================== */
    if is_new_month {
        tracing::info!("Monthly reset for user {}", user_id);

        sqlx::query("DELETE FROM user_sign_in_days WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM user_month_card_days WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    /* ===============================
     * STEP 2: Daily sign-in
     * =============================== */
    if is_new_day {
        let server_date = common::time::ServerTime::server_date();
        let day_of_month = server_date.day() as i32;
        let current_server_day = ServerTime::server_day(now);

        // Record daily sign-in
        sqlx::query(
            "INSERT INTO user_sign_in_days (user_id, server_day, day_of_month)
             VALUES (?, ?, ?)
             ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(current_server_day)
        .bind(day_of_month)
        .execute(pool)
        .await?;

        // Increment accumulated sign-in count
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
        .bind(ServerTime::now_ms())
        .execute(pool)
        .await?;

        // Month card daily credit
        let has_active_card: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM user_month_card_history
             WHERE user_id = ?
               AND start_time <= ?
               AND end_time > ?",
        )
        .bind(user_id)
        .bind(ServerTime::now_ms())
        .bind(ServerTime::now_ms())
        .fetch_optional(pool)
        .await?;

        if has_active_card.is_some() {
            sqlx::query(
                "INSERT INTO user_month_card_days (user_id, server_day)
                 VALUES (?, ?)
                 ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(current_server_day)
            .execute(pool)
            .await?;
        }

        tracing::info!("User {} logged in for day {}", user_id, day_of_month);
    }

    /* ===============================
     * STEP 3: Update timestamps
     * =============================== */
    sqlx::query(
        "UPDATE player_state
         SET last_sign_in_time = ?, updated_at = ?
         WHERE player_id = ?",
    )
    .bind(now as i64)
    .bind(now as i64)
    .bind(user_id)
    .execute(pool)
    .await?;

    sqlx::query("UPDATE users SET last_login_at = ?, updated_at = ? WHERE id = ?")
        .bind(now as i64)
        .bind(now as i64)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok((is_new_day, is_new_week, is_new_month))
}

pub async fn process_manual_sign_in(pool: &SqlitePool, user_id: i64) -> Result<()> {
    let now = ServerTime::now_ms();

    // Canonical server-day (monotonic, offset-aware)
    let server_day = ServerTime::server_day(now) as i32;

    // Also extract calendar fields if UI needs them
    let server_date = ServerTime::server_date();
    let day_of_month = server_date.day() as i32;

    // Check if already signed in today
    let already_signed: Option<i32> =
        sqlx::query_scalar("SELECT 1 FROM user_sign_in_days WHERE user_id = ? AND server_day = ?")
            .bind(user_id)
            .bind(server_day)
            .fetch_optional(pool)
            .await?;

    if already_signed.is_some() {
        tracing::info!("User {} already signed in today", user_id);
        return Ok(());
    }

    // Insert sign-in record (collision-safe)
    sqlx::query(
        "INSERT INTO user_sign_in_days (user_id, server_day, day_of_month)
         VALUES (?, ?, ?)
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(server_day)
    .bind(day_of_month)
    .execute(pool)
    .await?;

    //Increment accumulated sign-in count
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
    .bind(common::time::ServerTime::now_ms())
    .execute(pool)
    .await?;

    //Check month card validity (timestamps stay UTC ms)
    let has_active_card: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM user_month_card_history
         WHERE user_id = ?
           AND start_time <= ?
           AND end_time > ?",
    )
    .bind(user_id)
    .bind(now as i64)
    .bind(now as i64)
    .fetch_optional(pool)
    .await?;

    if has_active_card.is_some() {
        sqlx::query(
            "INSERT INTO user_month_card_days (user_id, server_day)
             VALUES (?, ?)
             ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(server_day)
        .execute(pool)
        .await?;
    }

    tracing::info!(
        "Manual sign-in recorded for user {} (server_day={}, day={})",
        user_id,
        server_day,
        day_of_month
    );

    Ok(())
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
pub async fn reset_weekly_counters(_pool: &SqlitePool, user_id: i64) -> Result<()> {
    // Add weekly reset logic here
    // Example: Reset tower attempts, weekly missions, etc.

    tracing::info!("Reset weekly counters for user {}", user_id);
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
    // ──────────────────────────────
    //  Main sign-in info
    // ──────────────────────────────
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

    // ──────────────────────────────
    // Signed-in days (DISPLAY ONLY)
    // ──────────────────────────────
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

    // ──────────────────────────────
    // Accumulated sign-in bonuses
    // ──────────────────────────────
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

    // ──────────────────────────────
    // Month card sign-in days (DISPLAY ONLY)
    // ──────────────────────────────
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

    // ──────────────────────────────
    // Month card history
    // ──────────────────────────────
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

    // ──────────────────────────────
    // Birthday heroes (derived)
    // ──────────────────────────────
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
    //Insert daily sign-in (collision-safe)
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

    // Increment accumulated sign-in count
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
