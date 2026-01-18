use crate::{
    models::game::heros::{HeroModel, UserHeroModel},
    models::game::dungeons::{
        DungeonLastHeroGroup, RewardPointInfo, UserChapterTypeNum, UserDungeon,
    },
};

use anyhow::Result;
use sqlx::{SqlitePool, prelude::FromRow};

pub async fn get_user_dungeons_chunked(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<Vec<UserDungeon>>> {
    let all_dungeons = get_user_dungeons(pool, user_id).await?;

    // Chunk size of 100 to match client limit
    let chunks: Vec<Vec<UserDungeon>> = all_dungeons
        .chunks(100)
        .map(|chunk| chunk.to_vec())
        .collect();

    Ok(chunks)
}

pub async fn get_user_dungeons(pool: &SqlitePool, user_id: i64) -> Result<Vec<UserDungeon>> {
    let dungeons = sqlx::query_as::<_, UserDungeon>(
        "SELECT * FROM user_dungeons WHERE user_id = ? ORDER BY chapter_id, episode_id
",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(dungeons)
}

#[derive(Debug, Clone, FromRow)]
pub struct UserDungeonInfo {
    pub star: i32,
    pub challenge_count: i32,
    pub has_record: bool,
}

pub async fn get_user_dungeon(
    pool: &SqlitePool,
    user_id: i64,
    chapter_id: i32,
    episode_id: i32,
) -> Result<UserDungeonInfo> {
    let dungeon = sqlx::query_as::<_, UserDungeonInfo>(
        "SELECT star, challenge_count, has_record
         FROM user_dungeons
         WHERE user_id = ? AND chapter_id = ? AND episode_id = ?",
    )
    .bind(user_id)
    .bind(chapter_id)
    .bind(episode_id)
    .fetch_one(pool)
    .await?;

    Ok(dungeon)
}

pub async fn get_dungeon_last_hero_groups(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<DungeonLastHeroGroup>> {
    // Get all last hero groups with their chapter IDs
    let rows: Vec<(i32, i32)> = sqlx::query_as(
        "SELECT chapter_id, hero_group_id FROM dungeon_last_hero_groups WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for (chapter_id, hero_group_id) in rows {
        // Get the hero group info
        if let Some(group_info) =
            crate::db::game::hero_groups::get_hero_group(pool, user_id, hero_group_id).await?
        {
            result.push(DungeonLastHeroGroup {
                chapter_id,
                hero_group_info: group_info,
            });
        }
    }

    Ok(result)
}

pub async fn get_unlocked_maps(pool: &SqlitePool, user_id: i64) -> Result<Vec<i32>> {
    let maps = sqlx::query_scalar("SELECT map_id FROM user_dungeon_maps WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(maps)
}

pub async fn get_elements(pool: &SqlitePool, user_id: i64) -> Result<Vec<i32>> {
    let elements = sqlx::query_scalar(
        "SELECT element_id FROM user_dungeon_elements WHERE user_id = ? AND is_finished = 0",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(elements)
}

pub async fn get_finished_elements(pool: &SqlitePool, user_id: i64) -> Result<Vec<i32>> {
    let elements = sqlx::query_scalar(
        "SELECT element_id FROM user_dungeon_elements WHERE user_id = ? AND is_finished = 1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(elements)
}

pub async fn get_reward_points(pool: &SqlitePool, user_id: i64) -> Result<Vec<RewardPointInfo>> {
    let points: Vec<(i32, i32)> = sqlx::query_as(
        "SELECT chapter_id, reward_point FROM user_dungeon_reward_points WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for (chapter_id, reward_point) in points {
        let claimed_rewards = sqlx::query_scalar(
            "SELECT point_reward_id FROM user_dungeon_claimed_rewards
             WHERE user_id = ? AND chapter_id = ?",
        )
        .bind(user_id)
        .bind(chapter_id)
        .fetch_all(pool)
        .await?;

        result.push(RewardPointInfo {
            chapter_id,
            reward_point,
            has_get_point_reward_ids: claimed_rewards,
        });
    }

    Ok(result)
}

pub async fn get_equip_sp_chapters(pool: &SqlitePool, user_id: i64) -> Result<Vec<i32>> {
    let chapters = sqlx::query_scalar(
        "SELECT chapter_id FROM user_dungeon_equip_sp_chapters WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(chapters)
}

pub async fn get_chapter_type_nums(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<UserChapterTypeNum>> {
    let nums = sqlx::query_as::<_, UserChapterTypeNum>(
        "SELECT chapter_type, today_pass_num, today_total_num
         FROM user_chapter_type_nums WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(nums)
}

pub async fn get_finished_puzzles(pool: &SqlitePool, user_id: i64) -> Result<Vec<i32>> {
    let puzzles =
        sqlx::query_scalar("SELECT puzzle_id FROM user_dungeon_finished_puzzles WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await?;
    Ok(puzzles)
}

pub async fn update_dungeon_progress(
    pool: &SqlitePool,
    user_id: i64,
    chapter_id: i32,
    episode_id: i32,
    stars_earned: i32,
) -> Result<()> {
    let now = common::time::ServerTime::now_ms();

    sqlx::query(
        r#"
        INSERT INTO user_dungeons
        (user_id, chapter_id, episode_id, star, challenge_count, has_record,
         left_return_all_num, today_pass_num, today_total_num, created_at, updated_at)
        VALUES (?, ?, ?, ?, 1, 1, 1, 1, 1, ?, ?)
        ON CONFLICT(user_id, chapter_id, episode_id) DO UPDATE SET
            star = CASE WHEN excluded.star > star THEN excluded.star ELSE star END,
            challenge_count = challenge_count + 1,
            has_record = 1,
            today_pass_num = today_pass_num + 1,
            today_total_num = today_total_num + 1,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(user_id)
    .bind(chapter_id)
    .bind(episode_id)
    .bind(stars_earned)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_dungeon_record(
    pool: &SqlitePool,
    user_id: i64,
    episode_id: i32,
) -> Result<Option<sonettobuf::FightGroupRecord>> {
    let hero = UserHeroModel::new(user_id, pool.clone());

    #[derive(sqlx::FromRow)]
    struct RecordRow {
        record_round: i32,
        hero_list: String,
        sub_hero_list: String,
        cloth_id: i32,
        equips: String,
        version: i32,
    }

    let row: Option<RecordRow> = sqlx::query_as(
        r#"
        SELECT record_round, hero_list, sub_hero_list, cloth_id, equips, version
        FROM dungeon_records
        WHERE user_id = ? AND episode_id = ?
        "#,
    )
    .bind(user_id)
    .bind(episode_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    // Parse hero UIDs and filter zeros
    let hero_uids: Vec<i64> = serde_json::from_str::<Vec<i64>>(&row.hero_list)?
        .into_iter()
        .filter(|&uid| uid != 0) // Filter zeros
        .collect();

    let sub_hero_uids: Vec<i64> = serde_json::from_str::<Vec<i64>>(&row.sub_hero_list)?
        .into_iter()
        .filter(|&uid| uid != 0) // Filter zeros
        .collect();

    // Build hero records
    let mut hero_list = Vec::new();
    for hero_uid in hero_uids {
        if let Ok(hero_data) = hero.get_uid(hero_uid as i32).await {
            hero_list.push(sonettobuf::FightHeroRecord {
                hero_uid: Some(hero_data.record.uid),
                hero_id: Some(hero_data.record.hero_id),
                level: Some(hero_data.record.level),
                skin: Some(hero_data.record.skin),
            });
        }
    }

    let mut sub_hero_list = Vec::new();
    for hero_uid in sub_hero_uids {
        if let Ok(hero_data) = hero.get_uid(hero_uid as i32).await {
            sub_hero_list.push(sonettobuf::FightHeroRecord {
                hero_uid: Some(hero_data.record.uid),
                hero_id: Some(hero_data.record.hero_id),
                level: Some(hero_data.record.level),
                skin: Some(hero_data.record.skin),
            });
        }
    }

    // Parse equipment data and filter zeros
    let all_equips: Vec<sonettobuf::FightEquipRecord> = serde_json::from_str(&row.equips)?;
    let equips: Vec<sonettobuf::FightEquipRecord> = all_equips
        .into_iter()
        .filter(|e| e.hero_uid.unwrap_or(0) != 0) // Filter zero hero UIDs
        .collect();

    Ok(Some(sonettobuf::FightGroupRecord {
        hero_list,
        sub_hero_list,
        cloth_id: Some(row.cloth_id),
        equips,
        trial_hero_list: vec![],
        activity104_equips: vec![],
        ex_infos: vec![],
        version: Some(row.version),
        assist_user_id: Some(0),
        assist_hero_uid: Some(0),
        record_round: Some(row.record_round),
        assist_boss_id: Some(0),
    }))
}

// Call this when battle completes successfully
pub async fn save_dungeon_record(
    pool: &SqlitePool,
    user_id: i64,
    episode_id: i32,
    record_round: i32,
    fight_group: &sonettobuf::FightGroup,
    equips: Vec<sonettobuf::FightEquipRecord>,
) -> Result<()> {
    let hero_list = serde_json::to_string(&fight_group.hero_list)?;
    let sub_hero_list = serde_json::to_string(&fight_group.sub_hero_list)?;
    let equips_json = serde_json::to_string(&equips)?;
    let cloth_id = fight_group.cloth_id.unwrap_or(1);

    sqlx::query(
        r#"
        INSERT INTO dungeon_records (user_id, episode_id, record_round, hero_list, sub_hero_list, cloth_id, equips, version, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 5, ?)
        ON CONFLICT(user_id, episode_id) DO UPDATE SET
            record_round = excluded.record_round,
            hero_list = excluded.hero_list,
            sub_hero_list = excluded.sub_hero_list,
            cloth_id = excluded.cloth_id,
            equips = excluded.equips,
            created_at = excluded.created_at
        "#,
    )
    .bind(user_id)
    .bind(episode_id)
    .bind(record_round)
    .bind(hero_list)
    .bind(sub_hero_list)
    .bind(cloth_id)
    .bind(equips_json)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn should_update_dungeon_record(
    pool: &SqlitePool,
    user_id: i64,
    episode_id: i32,
    new_round: i32,
    new_fight_group: &Option<sonettobuf::FightGroup>,
) -> Result<bool> {
    #[derive(sqlx::FromRow)]
    struct ExistingRecord {
        record_round: i32,
        hero_list: String,
        sub_hero_list: String,
    }

    // Check for existing record
    let existing: Option<ExistingRecord> = sqlx::query_as(
        "SELECT record_round, hero_list, sub_hero_list FROM dungeon_records
         WHERE user_id = ? AND episode_id = ?",
    )
    .bind(user_id)
    .bind(episode_id)
    .fetch_optional(pool)
    .await?;

    let Some(existing) = existing else {
        // No existing record, save this one
        return Ok(true);
    };

    // Save if new record is faster (fewer rounds)
    if new_round < existing.record_round {
        tracing::info!(
            "New record is faster: {} rounds vs {} rounds",
            new_round,
            existing.record_round
        );
        return Ok(true);
    }

    // If same or slower, check if lineup is different
    if new_round >= existing.record_round {
        // Get new lineup (main + sub heroes)
        let (new_main, new_sub) = new_fight_group
            .as_ref()
            .map(|fg| {
                let main: Vec<i64> = fg
                    .hero_list
                    .iter()
                    .filter(|&&uid| uid != 0)
                    .copied()
                    .collect();
                let sub: Vec<i64> = fg
                    .sub_hero_list
                    .iter()
                    .filter(|&&uid| uid != 0)
                    .copied()
                    .collect();
                (main, sub)
            })
            .unwrap_or_default();

        // Get existing lineup
        let existing_main: Vec<i64> = serde_json::from_str(&existing.hero_list).unwrap_or_default();
        let existing_sub: Vec<i64> =
            serde_json::from_str(&existing.sub_hero_list).unwrap_or_default();

        // Save if either main or sub lineup is different
        if new_main != existing_main || new_sub != existing_sub {
            tracing::info!(
                "Different lineup detected: main {:?} vs {:?}, sub {:?} vs {:?}",
                new_main,
                existing_main,
                new_sub,
                existing_sub
            );
            return Ok(true);
        }
    }

    // Don't save - existing record is better or same
    Ok(false)
}
