use crate::models::game::buildings::Building;
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn get_user_buildings(pool: &SqlitePool, user_id: i64) -> Result<Vec<Building>> {
    let buildings = sqlx::query_as::<_, Building>(
        "SELECT * FROM user_buildings WHERE user_id = ? ORDER BY uid",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(buildings)
}

pub async fn save_building(pool: &SqlitePool, building: &Building) -> Result<()> {
    let now = common::time::ServerTime::now_ms();

    sqlx::query(
        r#"
        INSERT INTO user_buildings (
            uid, user_id, define_id, in_use, x, y, rotate, level, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(uid) DO UPDATE SET
            define_id = excluded.define_id,
            in_use = excluded.in_use,
            x = excluded.x,
            y = excluded.y,
            rotate = excluded.rotate,
            level = excluded.level,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(building.uid)
    .bind(building.user_id)
    .bind(building.define_id)
    .bind(building.in_use)
    .bind(building.x)
    .bind(building.y)
    .bind(building.rotate)
    .bind(building.level)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_building_position(
    pool: &SqlitePool,
    uid: i64,
    x: i32,
    y: i32,
    rotate: i32,
) -> Result<()> {
    let now = common::time::ServerTime::now_ms();

    sqlx::query("UPDATE user_buildings SET x = ?, y = ?, rotate = ?, updated_at = ? WHERE uid = ?")
        .bind(x)
        .bind(y)
        .bind(rotate)
        .bind(now)
        .bind(uid)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn upgrade_building(pool: &SqlitePool, uid: i64) -> Result<()> {
    let now = common::time::ServerTime::now_ms();

    sqlx::query("UPDATE user_buildings SET level = level + 1, updated_at = ? WHERE uid = ?")
        .bind(now)
        .bind(uid)
        .execute(pool)
        .await?;

    Ok(())
}
