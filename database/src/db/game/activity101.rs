use anyhow::Result;
use common::time::ServerTime;
use sqlx::SqlitePool;

/// Get activity 101 info for a user
pub async fn get_activity101_info(
    pool: &SqlitePool,
    user_id: i64,
    activity_id: i32,
) -> Result<(Vec<(i32, i32)>, i32, bool)> {
    // Get claimed days
    let claimed_days: Vec<i32> = sqlx::query_scalar(
        "SELECT day_id FROM user_activity101_claims
         WHERE user_id = ? AND activity_id = ?
         ORDER BY day_id",
    )
    .bind(user_id)
    .bind(activity_id)
    .fetch_all(pool)
    .await?;

    // Get total login count (could use sign-in count or activity-specific count)
    let login_count = sqlx::query_scalar::<_, i32>(
        "SELECT addup_sign_in_day FROM user_sign_in_info WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or(0);

    // Get once bonus status
    let got_once_bonus = sqlx::query_scalar::<_, i32>(
        "SELECT 1 FROM user_activity101_once_bonus
         WHERE user_id = ? AND activity_id = ?",
    )
    .bind(user_id)
    .bind(activity_id)
    .fetch_optional(pool)
    .await?
    .is_some();

    // Build state for each day (1-7)
    let mut infos = Vec::new();
    for day in 1..=7 {
        let state = if claimed_days.contains(&day) {
            2 // Claimed
        } else if day <= login_count {
            1 // Available to claim
        } else {
            0 // Future day
        };
        infos.push((day, state));
    }

    Ok((infos, login_count, got_once_bonus))
}

pub async fn record_activity101_login(
    pool: &SqlitePool,
    user_id: i64,
    activity_id: i32,
) -> Result<()> {
    let server_day = ServerTime::server_day(ServerTime::now_ms());

    sqlx::query(
        "INSERT OR IGNORE INTO user_activity101_logins
         (user_id, activity_id, server_day, login_date)
         VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(activity_id)
    .bind(server_day)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;

    Ok(())
}

/// Claim a day's reward
pub async fn claim_activity101_day(
    pool: &SqlitePool,
    user_id: i64,
    activity_id: i32,
    day_id: i32,
) -> Result<bool> {
    let now = common::time::ServerTime::now_ms();

    let rows = sqlx::query(
        "INSERT INTO user_activity101_claims (user_id, activity_id, day_id, claimed_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(activity_id)
    .bind(day_id)
    .bind(now)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(rows > 0)
}
