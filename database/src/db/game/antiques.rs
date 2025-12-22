use crate::models::game::antiques::UserAntique;
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn get_user_antiques(pool: &SqlitePool, user_id: i64) -> Result<Vec<UserAntique>> {
    let antiques = sqlx::query_as::<_, UserAntique>(
        "SELECT user_id, antique_id, get_time FROM user_antiques WHERE user_id = ? ORDER BY antique_id"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(antiques)
}

pub async fn add_antique(pool: &SqlitePool, user_id: i64, antique_id: i32) -> Result<()> {
    let get_time = common::time::ServerTime::now_ms();

    sqlx::query(
        "INSERT INTO user_antiques (user_id, antique_id, get_time) VALUES (?, ?, ?) ON CONFLICT DO NOTHING"
    )
    .bind(user_id)
    .bind(antique_id)
    .bind(get_time)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn has_antique(pool: &SqlitePool, user_id: i64, antique_id: i32) -> Result<bool> {
    let exists: Option<i32> =
        sqlx::query_scalar("SELECT 1 FROM user_antiques WHERE user_id = ? AND antique_id = ?")
            .bind(user_id)
            .bind(antique_id)
            .fetch_optional(pool)
            .await?;
    Ok(exists.is_some())
}
