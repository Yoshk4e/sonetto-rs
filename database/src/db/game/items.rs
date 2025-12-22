use crate::models::game::items::{InsightItem, Item, PowerItem};
use common::time::ServerTime;
use sqlx::SqlitePool;
// Items
pub async fn get_all_items(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<Item>> {
    sqlx::query_as("SELECT * FROM items WHERE user_id = ? ORDER BY item_id")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn get_item(pool: &SqlitePool, user_id: i64, item_id: u32) -> sqlx::Result<Option<Item>> {
    sqlx::query_as("SELECT * FROM items WHERE user_id = ? AND item_id = ?")
        .bind(user_id)
        .bind(item_id as i64)
        .fetch_optional(pool)
        .await
}

pub async fn save_item(pool: &SqlitePool, item: &Item) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO items (user_id, item_id, quantity, last_use_time, last_update_time, total_gain_count)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id, item_id) DO UPDATE SET
             quantity = excluded.quantity,
             last_use_time = excluded.last_use_time,
             last_update_time = excluded.last_update_time,
             total_gain_count = excluded.total_gain_count"
    )
    .bind(item.user_id)
    .bind(item.item_id)
    .bind(item.quantity)
    .bind(item.last_use_time)
    .bind(item.last_update_time)
    .bind(item.total_gain_count)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_item_quantity(
    pool: &SqlitePool,
    user_id: i64,
    item_id: u32,
    amount: i32,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO items (user_id, item_id, quantity, last_update_time, total_gain_count)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(user_id, item_id) DO UPDATE SET
             quantity = quantity + excluded.quantity,
             last_update_time = excluded.last_update_time,
             total_gain_count = total_gain_count + excluded.total_gain_count",
    )
    .bind(user_id)
    .bind(item_id as i64)
    .bind(amount)
    .bind(ServerTime::now_ms())
    .bind(amount as i64)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_item_quantity(
    pool: &SqlitePool,
    user_id: i64,
    item_id: u32,
    amount: i32,
) -> sqlx::Result<bool> {
    let current: Option<i32> =
        sqlx::query_scalar("SELECT quantity FROM items WHERE user_id = ? AND item_id = ?")
            .bind(user_id)
            .bind(item_id as i64)
            .fetch_optional(pool)
            .await?;

    if current.unwrap_or(0) < amount {
        return Ok(false);
    }

    let timestamp = ServerTime::now_ms();
    sqlx::query("UPDATE items SET quantity = quantity - ?, last_use_time = ?, last_update_time = ? WHERE user_id = ? AND item_id = ?")
        .bind(amount)
        .bind(timestamp as i64)
        .bind(timestamp as i64)
        .bind(user_id)
        .bind(item_id as i64)
        .execute(pool)
        .await?;

    Ok(true)
}

// Power Items
pub async fn get_all_power_items(pool: &SqlitePool, user_id: i64) -> sqlx::Result<Vec<PowerItem>> {
    sqlx::query_as("SELECT * FROM power_items WHERE user_id = ? AND expire_time > strftime('%s', 'now') ORDER BY expire_time")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn insert_power_item(pool: &SqlitePool, item: &PowerItem) -> sqlx::Result<i64> {
    let result = sqlx::query(
        "INSERT INTO power_items (user_id, item_id, quantity, expire_time) VALUES (?, ?, ?, ?)",
    )
    .bind(item.user_id)
    .bind(item.item_id)
    .bind(item.quantity)
    .bind(item.expire_time)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn delete_power_item(pool: &SqlitePool, user_id: i64, uid: i64) -> sqlx::Result<bool> {
    let result = sqlx::query("DELETE FROM power_items WHERE user_id = ? AND uid = ?")
        .bind(user_id)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn cleanup_expired_power_items(
    pool: &SqlitePool,
    user_id: Option<i64>,
) -> sqlx::Result<u64> {
    let result = match user_id {
        Some(uid) => sqlx::query(
            "DELETE FROM power_items WHERE user_id = ? AND expire_time <= strftime('%s', 'now')",
        )
        .bind(uid)
        .execute(pool)
        .await?,
        None => {
            sqlx::query("DELETE FROM power_items WHERE expire_time <= strftime('%s', 'now')")
                .execute(pool)
                .await?
        }
    };
    Ok(result.rows_affected())
}

// Insight Items
pub async fn get_all_insight_items(
    pool: &SqlitePool,
    user_id: i64,
) -> sqlx::Result<Vec<InsightItem>> {
    sqlx::query_as("SELECT * FROM insight_items WHERE user_id = ? AND expire_time > strftime('%s', 'now') ORDER BY expire_time")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn insert_insight_item(pool: &SqlitePool, item: &InsightItem) -> sqlx::Result<i64> {
    let result = sqlx::query(
        "INSERT INTO insight_items (user_id, item_id, quantity, expire_time) VALUES (?, ?, ?, ?)",
    )
    .bind(item.user_id)
    .bind(item.item_id)
    .bind(item.quantity)
    .bind(item.expire_time)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn delete_insight_item(pool: &SqlitePool, user_id: i64, uid: i64) -> sqlx::Result<bool> {
    let result = sqlx::query("DELETE FROM insight_items WHERE user_id = ? AND uid = ?")
        .bind(user_id)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn cleanup_expired_insight_items(
    pool: &SqlitePool,
    user_id: Option<i64>,
) -> sqlx::Result<u64> {
    let result = match user_id {
        Some(uid) => sqlx::query(
            "DELETE FROM insight_items WHERE user_id = ? AND expire_time <= strftime('%s', 'now')",
        )
        .bind(uid)
        .execute(pool)
        .await?,
        None => {
            sqlx::query("DELETE FROM insight_items WHERE expire_time <= strftime('%s', 'now')")
                .execute(pool)
                .await?
        }
    };
    Ok(result.rows_affected())
}
