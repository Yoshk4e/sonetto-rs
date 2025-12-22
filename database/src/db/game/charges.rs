use crate::models::game::charges::{ChargeInfo, SandboxSettings};
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn get_charge_infos(pool: &SqlitePool, user_id: i64) -> Result<Vec<ChargeInfo>> {
    let infos = sqlx::query_as::<_, ChargeInfo>(
        "SELECT * FROM user_charge_info WHERE user_id = ? ORDER BY charge_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(infos)
}

pub async fn get_sandbox_settings(pool: &SqlitePool, user_id: i64) -> Result<SandboxSettings> {
    let settings = sqlx::query_as::<_, SandboxSettings>(
        "SELECT * FROM user_sandbox_settings WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(settings.unwrap_or(SandboxSettings {
        user_id,
        sandbox_enable: false,
        sandbox_balance: 0,
    }))
}

pub async fn record_purchase(pool: &SqlitePool, user_id: i64, charge_id: i32) -> Result<()> {
    let now = common::time::ServerTime::now_ms();

    sqlx::query(
        r#"
        INSERT INTO user_charge_info (user_id, charge_id, buy_count, first_charge, created_at, updated_at)
        VALUES (?, ?, 1, 0, ?, ?)
        ON CONFLICT(user_id, charge_id) DO UPDATE SET
            buy_count = buy_count + 1,
            first_charge = 0,
            updated_at = excluded.updated_at
        "#
    )
    .bind(user_id)
    .bind(charge_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_sandbox_balance(pool: &SqlitePool, user_id: i64, balance: i32) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO user_sandbox_settings (user_id, sandbox_enable, sandbox_balance)
        VALUES (?, 1, ?)
        ON CONFLICT(user_id) DO UPDATE SET
            sandbox_enable = 1,
            sandbox_balance = excluded.sandbox_balance
        "#,
    )
    .bind(user_id)
    .bind(balance)
    .execute(pool)
    .await?;

    Ok(())
}
