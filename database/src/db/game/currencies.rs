use crate::models::game::currencies::Currency;
use common::time::ServerTime;
use sqlx::SqlitePool;

pub async fn get_currencies(
    pool: &SqlitePool,
    user_id: i64,
    currency_ids: &[i32],
) -> sqlx::Result<Vec<Currency>> {
    if currency_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = currency_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "SELECT user_id, currency_id, quantity, last_recover_time, expired_time
         FROM currencies
         WHERE user_id = ? AND currency_id IN ({})
         ORDER BY currency_id",
        placeholders
    );

    let mut q = sqlx::query_as::<_, Currency>(&query).bind(user_id);
    for id in currency_ids {
        q = q.bind(id);
    }

    q.fetch_all(pool).await
}

pub async fn get_currency(
    pool: &SqlitePool,
    user_id: i64,
    currency_id: i32,
) -> sqlx::Result<Option<Currency>> {
    sqlx::query_as::<_, Currency>(
        "SELECT user_id, currency_id, quantity, last_recover_time, expired_time
         FROM currencies
         WHERE user_id = ? AND currency_id = ?",
    )
    .bind(user_id)
    .bind(currency_id)
    .fetch_optional(pool)
    .await
}

pub async fn save_currency(pool: &SqlitePool, currency: &Currency) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO currencies (user_id, currency_id, quantity, last_recover_time, expired_time)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(user_id, currency_id) DO UPDATE SET
             quantity = excluded.quantity,
             last_recover_time = excluded.last_recover_time,
             expired_time = excluded.expired_time",
    )
    .bind(currency.user_id)
    .bind(currency.currency_id)
    .bind(currency.quantity)
    .bind(currency.last_recover_time)
    .bind(currency.expired_time)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_currency(
    pool: &SqlitePool,
    user_id: i64,
    currency_id: i32,
    amount: i32,
) -> sqlx::Result<()> {
    let timestamp = ServerTime::now_ms();

    sqlx::query(
        "INSERT INTO currencies (user_id, currency_id, quantity, last_recover_time, expired_time)
         VALUES (?, ?, ?, ?, 0)
         ON CONFLICT(user_id, currency_id) DO UPDATE SET
             quantity = quantity + excluded.quantity,
             last_recover_time = excluded.last_recover_time",
    )
    .bind(user_id)
    .bind(currency_id)
    .bind(amount)
    .bind(timestamp as i64)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_currency(
    pool: &SqlitePool,
    user_id: i64,
    currency_id: i32,
    amount: i32,
) -> sqlx::Result<bool> {
    let current: Option<i32> =
        sqlx::query_scalar("SELECT quantity FROM currencies WHERE user_id = ? AND currency_id = ?")
            .bind(user_id)
            .bind(currency_id)
            .fetch_optional(pool)
            .await?;

    if current.unwrap_or(0) < amount {
        return Ok(false);
    }

    let timestamp = ServerTime::now_ms();
    sqlx::query(
        "UPDATE currencies
         SET quantity = quantity - ?, last_recover_time = ?
         WHERE user_id = ? AND currency_id = ?",
    )
    .bind(amount)
    .bind(timestamp as i64)
    .bind(user_id)
    .bind(currency_id)
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn set_currency(
    pool: &SqlitePool,
    user_id: i64,
    currency_id: i32,
    quantity: i32,
) -> sqlx::Result<()> {
    let timestamp = ServerTime::now_ms();

    sqlx::query(
        "INSERT INTO currencies (user_id, currency_id, quantity, last_recover_time, expired_time)
         VALUES (?, ?, ?, ?, 0)
         ON CONFLICT(user_id, currency_id) DO UPDATE SET
             quantity = excluded.quantity,
             last_recover_time = excluded.last_recover_time",
    )
    .bind(user_id)
    .bind(currency_id)
    .bind(quantity)
    .bind(timestamp as i64)
    .execute(pool)
    .await?;
    Ok(())
}
