use crate::models::game::block_packages::SpecialBlock;
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn get_block_packages(pool: &SqlitePool, user_id: i64) -> Result<Vec<i32>> {
    let packages = sqlx::query_scalar(
        "SELECT block_package_id FROM user_block_packages WHERE user_id = ? ORDER BY block_package_id"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(packages)
}

pub async fn get_special_blocks(pool: &SqlitePool, user_id: i64) -> Result<Vec<SpecialBlock>> {
    let blocks = sqlx::query_as::<_, SpecialBlock>(
        "SELECT user_id, block_id, create_time FROM user_special_blocks WHERE user_id = ? ORDER BY block_id"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(blocks)
}

pub async fn add_block_package(pool: &SqlitePool, user_id: i64, package_id: i32) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_block_packages (user_id, block_package_id) VALUES (?, ?) ON CONFLICT DO NOTHING"
    )
    .bind(user_id)
    .bind(package_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn add_special_block(pool: &SqlitePool, user_id: i64, block_id: i32) -> Result<()> {
    let create_time = common::time::ServerTime::now_ms();

    sqlx::query(
        "INSERT INTO user_special_blocks (user_id, block_id, create_time) VALUES (?, ?, ?) ON CONFLICT DO NOTHING"
    )
    .bind(user_id)
    .bind(block_id)
    .bind(create_time)
    .execute(pool)
    .await?;
    Ok(())
}
