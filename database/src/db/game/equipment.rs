use anyhow::Result;
use sonettobuf::FightEquipRecord;
use sqlx::SqlitePool;

pub use crate::models::game::equipment::Equipment;

/// Get all equipment for a user
pub async fn get_user_equipment(pool: &SqlitePool, user_id: i64) -> Result<Vec<Equipment>> {
    let equipment = sqlx::query_as::<_, Equipment>(
        "SELECT * FROM equipment WHERE user_id = ?1 AND count > 0 ORDER BY equip_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(equipment)
}

pub async fn get_hero_default_equip_id(
    pool: &SqlitePool,
    hero_uid: i64,
    user_id: i64,
) -> Result<Option<i32>> {
    let equip_id: Option<i32> = sqlx::query_scalar(
        r#"
        SELECT e.equip_id
        FROM heroes h
        LEFT JOIN equipment e
          ON e.uid = h.default_equip_uid
        WHERE h.uid = ? AND h.user_id = ?
        "#,
    )
    .bind(hero_uid)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(equip_id)
}

pub async fn get_equipment_by_uid(
    pool: &SqlitePool,
    user_id: i64,
    equip_uid: i64,
) -> Result<Equipment> {
    let equip = sqlx::query_as::<_, Equipment>(
        r#"
        SELECT uid, user_id, equip_id, level, exp, break_lv, count, is_lock, refine_lv, created_at, updated_at
        FROM equipment
        WHERE uid = ? AND user_id = ?
        "#,
    )
    .bind(equip_uid)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(equip)
}

pub async fn update_equipment_lock(
    pool: &SqlitePool,
    user_id: i64,
    uid: i64,
    is_lock: bool,
) -> Result<bool> {
    let now = common::time::ServerTime::now_ms();

    let rows_affected = sqlx::query(
        "UPDATE equipment SET is_lock = ?, updated_at = ? WHERE uid = ? AND user_id = ?",
    )
    .bind(is_lock)
    .bind(now)
    .bind(uid)
    .bind(user_id)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
}

pub async fn build_equip_records(
    pool: &SqlitePool,
    player_id: i64,
    fight_group: &Option<sonettobuf::FightGroup>,
) -> Result<Vec<FightEquipRecord>> {
    let Some(fg) = fight_group else {
        return Ok(vec![]);
    };

    let mut equip_records = Vec::new();

    for equip in &fg.equips {
        let hero_uid = equip.hero_uid.unwrap_or(0);
        let mut records = Vec::new();

        for &equip_uid in &equip.equip_uid {
            if equip_uid == 0 {
                continue;
            }

            if let Ok(equip_data) = get_equipment_by_uid(pool, player_id, equip_uid).await {
                records.push(sonettobuf::EquipRecord {
                    equip_uid: Some(equip_uid),
                    equip_id: Some(equip_data.equip_id),
                    equip_lv: Some(equip_data.level),
                    refine_lv: Some(equip_data.refine_lv),
                });
            }
        }

        equip_records.push(FightEquipRecord {
            hero_uid: Some(hero_uid),
            equip_records: records,
        });
    }

    Ok(equip_records)
}

pub async fn add_equipment(
    pool: &SqlitePool,
    user_id: i64,
    equip_id: i32,
    count: i32,
) -> Result<Vec<i64>> {
    let now = common::time::ServerTime::now_ms();
    let game_data = data::exceldb::get();
    let equip = game_data
        .equip
        .get(equip_id)
        .ok_or_else(|| anyhow::anyhow!("Equipment {} not found", equip_id))?;

    let (level, break_lv, refine_lv, mut is_lock) = match equip.rare {
        5 => (1, 0, 1, true),
        4 => (1, 0, 1, true),
        _ => (1, 0, 1, false),
    };

    if matches!(equip.name_en.as_str(), "Enlighten" | "Gluttony" | "Greed") {
        is_lock = false;
    }

    let is_stackable = matches!(equip_id, 1002 | 1003 | 1004 | 1005);

    let mut uids = Vec::new();

    if is_stackable {
        // Try to find existing stack
        if let Some(uid) = sqlx::query_scalar::<_, i64>(
            "SELECT uid FROM equipment WHERE user_id = ? AND equip_id = ? LIMIT 1",
        )
        .bind(user_id)
        .bind(equip_id)
        .fetch_optional(pool)
        .await?
        {
            sqlx::query(
                r#"
                UPDATE equipment
                SET count = count + ?, updated_at = ?
                WHERE uid = ? AND user_id = ?
                "#,
            )
            .bind(count)
            .bind(now)
            .bind(uid)
            .bind(user_id)
            .execute(pool)
            .await?;

            uids.push(uid);
        } else {
            let uid = sqlx::query_scalar::<_, i64>(
                "SELECT COALESCE(MAX(uid), 29999999) + 1 FROM equipment WHERE user_id = ?",
            )
            .bind(user_id)
            .fetch_one(pool)
            .await?;

            sqlx::query(
                r#"
                INSERT INTO equipment
                  (uid, user_id, equip_id, level, exp, break_lv, count, is_lock, refine_lv, created_at, updated_at)
                VALUES
                  (?, ?, ?, ?, 0, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(uid)
            .bind(user_id)
            .bind(equip_id)
            .bind(level)
            .bind(break_lv)
            .bind(count)
            .bind(is_lock)
            .bind(refine_lv)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

            uids.push(uid);
        }
    } else {
        let mut next_uid = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(uid), 29999999) + 1 FROM equipment WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        for _ in 0..count {
            sqlx::query(
                r#"
                INSERT INTO equipment
                  (uid, user_id, equip_id, level, exp, break_lv, count, is_lock, refine_lv, created_at, updated_at)
                VALUES
                  (?, ?, ?, ?, 0, ?, 1, ?, ?, ?, ?)
                "#,
            )
            .bind(next_uid)
            .bind(user_id)
            .bind(equip_id)
            .bind(level)
            .bind(break_lv)
            .bind(is_lock)
            .bind(refine_lv)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await?;

            uids.push(next_uid);
            next_uid += 1;
        }
    }

    Ok(uids)
}

/// Get total count of equipment by equip_id (counts all matching rows)
pub async fn get_equipment_count(pool: &SqlitePool, user_id: i64, equip_id: i32) -> Result<i32> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM equipment WHERE user_id = ? AND equip_id = ?")
            .bind(user_id)
            .bind(equip_id)
            .fetch_one(pool)
            .await?;

    Ok(count as i32)
}

pub async fn update_equipment_count(
    pool: &SqlitePool,
    user_id: i64,
    equip_id: i32,
    amount: i32,
) -> Result<Vec<i64>> {
    let now = common::time::ServerTime::now_ms();

    let uid = sqlx::query_scalar::<_, i64>(
        "SELECT uid FROM equipment WHERE user_id = ? AND equip_id = ? LIMIT 1",
    )
    .bind(user_id)
    .bind(equip_id)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE equipment
        SET count = count + ?, updated_at = ?
        WHERE uid = ? AND user_id = ?
        "#,
    )
    .bind(amount)
    .bind(now)
    .bind(uid)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(vec![uid])
}

pub async fn add_equipments(
    pool: &SqlitePool,
    user_id: i64,
    equips: &[(i32, i32)],
) -> Result<Vec<i64>> {
    let mut changed_uids = Vec::new();

    for (equip_id, count) in equips {
        let is_stackable = matches!(*equip_id, 1002 | 1003 | 1004 | 1005);

        if is_stackable {
            if let Some(uid) = sqlx::query_scalar::<_, i64>(
                "SELECT uid FROM equipment WHERE user_id = ? AND equip_id = ? LIMIT 1",
            )
            .bind(user_id)
            .bind(equip_id)
            .fetch_optional(pool)
            .await?
            {
                update_equipment_count(pool, user_id, *equip_id, *count).await?;
                changed_uids.push(uid);
            } else {
                let uids = add_equipment(pool, user_id, *equip_id, *count).await?;
                debug_assert_eq!(uids.len(), 1);
                changed_uids.push(uids[0]);
            }
        } else {
            let uids = add_equipment(pool, user_id, *equip_id, *count).await?;
            changed_uids.extend(uids);
        }
    }

    Ok(changed_uids)
}
