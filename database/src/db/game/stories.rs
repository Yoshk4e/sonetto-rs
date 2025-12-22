use crate::models::game::stories::ProcessingStory;
use anyhow::Result;
use sqlx::SqlitePool;

pub async fn get_finished_stories(pool: &SqlitePool, user_id: i64) -> Result<Vec<i32>> {
    let stories = sqlx::query_scalar(
        "SELECT story_id FROM user_finished_stories WHERE user_id = ? ORDER BY story_id"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(stories)
}

pub async fn get_processing_stories(pool: &SqlitePool, user_id: i64) -> Result<Vec<ProcessingStory>> {
    let stories = sqlx::query_as::<_, ProcessingStory>(
        "SELECT * FROM user_processing_stories WHERE user_id = ? ORDER BY story_id"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(stories)
}

pub async fn finish_story(pool: &SqlitePool, user_id: i64, story_id: i32) -> Result<()> {
    // Move from processing to finished
    sqlx::query("DELETE FROM user_processing_stories WHERE user_id = ? AND story_id = ?")
        .bind(user_id)
        .bind(story_id)
        .execute(pool)
        .await?;

    sqlx::query("INSERT INTO user_finished_stories (user_id, story_id) VALUES (?, ?) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(story_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_processing_story(
    pool: &SqlitePool,
    user_id: i64,
    story_id: i32,
    step_id: i32,
    favor: i32,
) -> Result<()> {
    let now = common::time::ServerTime::now_ms();

    sqlx::query(
        r#"
        INSERT INTO user_processing_stories (user_id, story_id, step_id, favor, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(user_id, story_id) DO UPDATE SET
            step_id = excluded.step_id,
            favor = excluded.favor,
            updated_at = excluded.updated_at
        "#
    )
    .bind(user_id)
    .bind(story_id)
    .bind(step_id)
    .bind(favor)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}
