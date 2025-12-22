use crate::models::game::hero_group_snapshots::{
    HeroGroupSnapshot, HeroGroupSnapshotGroup, HeroGroupSnapshotInfo,
};
use crate::models::game::hero_groups::{HeroGroupEquip, HeroGroupInfo};
use anyhow::Result;
use sqlx::SqlitePool;
use std::collections::HashMap;

/// Helper to build HeroGroupInfo from a snapshot group
async fn build_snapshot_group_info(
    pool: &SqlitePool,
    snapshot_group_id: i64,
    group_id: i32,
) -> Result<HeroGroupInfo> {
    // Get group details
    let group = sqlx::query_as::<_, HeroGroupSnapshotGroup>(
        "SELECT * FROM hero_group_snapshot_groups WHERE id = ?",
    )
    .bind(snapshot_group_id)
    .fetch_one(pool)
    .await?;

    // Get hero members
    let hero_list: Vec<i64> = sqlx::query_scalar(
        "SELECT hero_uid FROM hero_group_snapshot_members WHERE snapshot_group_id = ? ORDER BY position"
    )
    .bind(snapshot_group_id)
    .fetch_all(pool)
    .await?;

    // Get equips
    let equip_rows: Vec<(i32, i64)> = sqlx::query_as(
        "SELECT index_slot, equip_uid FROM hero_group_snapshot_equips WHERE snapshot_group_id = ? ORDER BY index_slot"
    )
    .bind(snapshot_group_id)
    .fetch_all(pool)
    .await?;

    let mut equips_map: HashMap<i32, Vec<i64>> = HashMap::new();
    for (index, equip_uid) in equip_rows {
        equips_map.entry(index).or_default().push(equip_uid);
    }

    let equips = equips_map
        .into_iter()
        .map(|(index, equip_uids)| HeroGroupEquip { index, equip_uids })
        .collect();

    // Get activity104 equips
    let activity104_rows: Vec<(i32, i64)> = sqlx::query_as(
        "SELECT index_slot, equip_uid FROM hero_group_snapshot_activity104_equips WHERE snapshot_group_id = ? ORDER BY index_slot"
    )
    .bind(snapshot_group_id)
    .fetch_all(pool)
    .await?;

    let mut activity104_map: HashMap<i32, Vec<i64>> = HashMap::new();
    for (index, equip_uid) in activity104_rows {
        activity104_map.entry(index).or_default().push(equip_uid);
    }

    let activity104_equips = activity104_map
        .into_iter()
        .map(|(index, equip_uids)| HeroGroupEquip { index, equip_uids })
        .collect();

    tracing::info!(
        "Loaded group {}: sub {} {} heroes: {:?}",
        group_id,
        snapshot_group_id,
        hero_list.len(),
        hero_list
    );

    Ok(HeroGroupInfo {
        group_id,
        hero_list,
        name: group.name,
        cloth_id: group.cloth_id,
        equips,
        activity104_equips,
        assist_boss_id: group.assist_boss_id,
    })
}

/// Get all snapshots for a user
pub async fn get_hero_group_snapshots(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<HeroGroupSnapshotInfo>> {
    let snapshots = sqlx::query_as::<_, HeroGroupSnapshot>(
        "SELECT * FROM hero_group_snapshots WHERE user_id = ? ORDER BY snapshot_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();

    for snapshot in snapshots {
        // Get all groups in this snapshot
        let snapshot_groups = sqlx::query_as::<_, HeroGroupSnapshotGroup>(
            "SELECT * FROM hero_group_snapshot_groups WHERE snapshot_id = ? ORDER BY group_id",
        )
        .bind(snapshot.id)
        .fetch_all(pool)
        .await?;

        let mut hero_group_snapshots = Vec::new();
        for group in snapshot_groups {
            let info = build_snapshot_group_info(pool, group.id, group.group_id).await?;
            hero_group_snapshots.push(info);
        }

        // Get sort sub IDs
        let sort_sub_ids: Vec<i32> = sqlx::query_scalar(
            "SELECT sub_id FROM hero_group_snapshot_sort_ids WHERE snapshot_id = ? ORDER BY sort_order"
        )
        .bind(snapshot.id)
        .fetch_all(pool)
        .await?;

        result.push(HeroGroupSnapshotInfo {
            snapshot_id: snapshot.snapshot_id,
            hero_group_snapshots,
            sort_sub_ids,
        });
    }

    Ok(result)
}

/// Get a specific snapshot by ID
pub async fn get_hero_group_snapshot(
    pool: &SqlitePool,
    user_id: i64,
    snapshot_id: i32,
) -> Result<Option<HeroGroupSnapshotInfo>> {
    let snapshot = sqlx::query_as::<_, HeroGroupSnapshot>(
        "SELECT * FROM hero_group_snapshots WHERE user_id = ? AND snapshot_id = ?",
    )
    .bind(user_id)
    .bind(snapshot_id)
    .fetch_optional(pool)
    .await?;

    let Some(snapshot) = snapshot else {
        return Ok(None);
    };

    // Get all groups in this snapshot
    let snapshot_groups = sqlx::query_as::<_, HeroGroupSnapshotGroup>(
        "SELECT * FROM hero_group_snapshot_groups WHERE snapshot_id = ? ORDER BY group_id",
    )
    .bind(snapshot.id)
    .fetch_all(pool)
    .await?;

    let mut hero_group_snapshots = Vec::new();
    for group in snapshot_groups {
        let info = build_snapshot_group_info(pool, group.id, group.group_id).await?;
        hero_group_snapshots.push(info);
    }

    // Get sort sub IDs
    let sort_sub_ids: Vec<i32> = sqlx::query_scalar(
        "SELECT sub_id FROM hero_group_snapshot_sort_ids WHERE snapshot_id = ? ORDER BY sort_order",
    )
    .bind(snapshot.id)
    .fetch_all(pool)
    .await?;

    Ok(Some(HeroGroupSnapshotInfo {
        snapshot_id: snapshot.snapshot_id,
        hero_group_snapshots,
        sort_sub_ids,
    }))
}

/// Save a snapshot from current hero groups
pub async fn save_hero_group_snapshot(
    pool: &SqlitePool,
    user_id: i64,
    snapshot_id: i32,
    groups: Vec<HeroGroupInfo>,
    sort_sub_ids: Vec<i32>,
) -> Result<()> {
    let now = common::time::ServerTime::now_ms();

    // Create or update snapshot
    sqlx::query(
        "INSERT INTO hero_group_snapshots (user_id, snapshot_id, created_at, updated_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(user_id, snapshot_id) DO UPDATE SET updated_at = excluded.updated_at",
    )
    .bind(user_id)
    .bind(snapshot_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // Get the snapshot DB ID
    let db_snapshot_id: i64 = sqlx::query_scalar(
        "SELECT id FROM hero_group_snapshots WHERE user_id = ? AND snapshot_id = ?",
    )
    .bind(user_id)
    .bind(snapshot_id)
    .fetch_one(pool)
    .await?;

    // Save each group (only delete the specific group being updated)
    for group in groups {
        // Delete only THIS specific group and its related data
        let existing_group: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM hero_group_snapshot_groups
             WHERE snapshot_id = ? AND group_id = ?",
        )
        .bind(db_snapshot_id)
        .bind(group.group_id)
        .fetch_optional(pool)
        .await?;

        if let Some(old_group_id) = existing_group {
            // Delete old members
            sqlx::query("DELETE FROM hero_group_snapshot_members WHERE snapshot_group_id = ?")
                .bind(old_group_id)
                .execute(pool)
                .await?;

            // Delete old equips
            sqlx::query("DELETE FROM hero_group_snapshot_equips WHERE snapshot_group_id = ?")
                .bind(old_group_id)
                .execute(pool)
                .await?;

            // Delete old activity104 equips
            sqlx::query(
                "DELETE FROM hero_group_snapshot_activity104_equips WHERE snapshot_group_id = ?",
            )
            .bind(old_group_id)
            .execute(pool)
            .await?;

            // Delete the group itself
            sqlx::query("DELETE FROM hero_group_snapshot_groups WHERE id = ?")
                .bind(old_group_id)
                .execute(pool)
                .await?;
        }

        // Insert new group
        let group_result = sqlx::query(
            "INSERT INTO hero_group_snapshot_groups (snapshot_id, group_id, name, cloth_id, assist_boss_id)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(db_snapshot_id)
        .bind(group.group_id)
        .bind(&group.name)
        .bind(group.cloth_id)
        .bind(group.assist_boss_id)
        .execute(pool)
        .await?;

        let snapshot_group_id = group_result.last_insert_rowid();

        // Save heroes
        for (position, hero_uid) in group.hero_list.iter().enumerate() {
            sqlx::query(
                "INSERT INTO hero_group_snapshot_members (snapshot_group_id, hero_uid, position)
                 VALUES (?, ?, ?)",
            )
            .bind(snapshot_group_id)
            .bind(hero_uid)
            .bind(position as i32)
            .execute(pool)
            .await?;
        }

        // Save equips
        for equip in &group.equips {
            for equip_uid in &equip.equip_uids {
                sqlx::query(
                    "INSERT INTO hero_group_snapshot_equips (snapshot_group_id, index_slot, equip_uid)
                     VALUES (?, ?, ?)"
                )
                .bind(snapshot_group_id)
                .bind(equip.index)
                .bind(equip_uid)
                .execute(pool)
                .await?;
            }
        }

        // Save activity104 equips
        for equip in &group.activity104_equips {
            for equip_uid in &equip.equip_uids {
                sqlx::query(
                    "INSERT INTO hero_group_snapshot_activity104_equips (snapshot_group_id, index_slot, equip_uid)
                     VALUES (?, ?, ?)"
                )
                .bind(snapshot_group_id)
                .bind(equip.index)
                .bind(equip_uid)
                .execute(pool)
                .await?;
            }
        }
    }

    // Update sort IDs - merge with existing ones
    let existing_sort_ids: Vec<i32> = sqlx::query_scalar(
        "SELECT sub_id FROM hero_group_snapshot_sort_ids
         WHERE snapshot_id = ? ORDER BY sort_order",
    )
    .bind(db_snapshot_id)
    .fetch_all(pool)
    .await?;

    // Merge: add new sort_sub_ids if not already present
    let mut merged_sort_ids = existing_sort_ids;
    for sub_id in &sort_sub_ids {
        if !merged_sort_ids.contains(sub_id) {
            merged_sort_ids.push(*sub_id);
        }
    }

    // Replace all sort IDs
    sqlx::query("DELETE FROM hero_group_snapshot_sort_ids WHERE snapshot_id = ?")
        .bind(db_snapshot_id)
        .execute(pool)
        .await?;

    for (order, sub_id) in merged_sort_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO hero_group_snapshot_sort_ids (snapshot_id, sub_id, sort_order)
             VALUES (?, ?, ?)",
        )
        .bind(db_snapshot_id)
        .bind(sub_id)
        .bind(order as i32)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn sync_snapshot_to_common(
    pool: &SqlitePool,
    user_id: i64,
    group: &HeroGroupInfo,
) -> Result<()> {
    let db_group_id: i64 =
        sqlx::query_scalar("SELECT id FROM hero_groups_common WHERE user_id = ? AND group_id = ?")
            .bind(user_id)
            .bind(group.group_id)
            .fetch_one(pool)
            .await?;

    // Replace heroes
    sqlx::query("DELETE FROM hero_group_members WHERE hero_group_id = ?")
        .bind(db_group_id)
        .execute(pool)
        .await?;

    for (pos, hero_uid) in group.hero_list.iter().enumerate() {
        sqlx::query(
            "INSERT INTO hero_group_members (hero_group_id, hero_uid, position)
             VALUES (?, ?, ?)",
        )
        .bind(db_group_id)
        .bind(hero_uid)
        .bind(pos as i32)
        .execute(pool)
        .await?;
    }

    // Replace equips
    sqlx::query("DELETE FROM hero_group_equips WHERE hero_group_id = ?")
        .bind(db_group_id)
        .execute(pool)
        .await?;

    for equip in &group.equips {
        for uid in &equip.equip_uids {
            sqlx::query(
                "INSERT INTO hero_group_equips (hero_group_id, index_slot, equip_uid)
                 VALUES (?, ?, ?)",
            )
            .bind(db_group_id)
            .bind(equip.index)
            .bind(uid)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}
