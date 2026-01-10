use crate::error::AppError;
use crate::state::ConnectionContext;
use database::db::game::{currencies, items, red_dots};
use sonettobuf::{
    CmdId, CurrencyChangePush, EndDungeonPush, ItemChangePush, MaterialChangePush, MaterialData,
    UpdateRedDotPush,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn send_red_dot_push(
    ctx: Arc<Mutex<ConnectionContext>>,
    user_id: i64,
    define_ids: Option<Vec<i32>>,
) -> Result<(), AppError> {
    let red_dot_records = {
        let ctx_guard = ctx.lock().await;
        match define_ids {
            Some(ids) => {
                red_dots::red_dots::get_red_dots_by_defines(&ctx_guard.state.db, user_id, &ids)
                    .await?
            }
            None => red_dots::red_dots::get_red_dots(&ctx_guard.state.db, user_id).await?,
        }
    };

    if !red_dot_records.is_empty() {
        let groups = red_dots::red_dots::group_red_dots(red_dot_records);
        let mut ctx_guard = ctx.lock().await;
        let push = UpdateRedDotPush {
            red_dot_infos: groups.into_iter().map(Into::into).collect(),
            replace_all: Some(true),
        };
        ctx_guard
            .send_push(CmdId::UpdateRedDotPushCmd, push)
            .await?;
    }

    Ok(())
}

pub async fn send_item_change_push(
    ctx: Arc<Mutex<ConnectionContext>>,
    user_id: i64,
    changed_item_ids: Vec<u32>,
    changed_power_item_ids: Vec<u32>,
    changed_insight_item_ids: Vec<u32>,
) -> Result<(), AppError> {
    let (items_list, power_items_list, insight_items_list) = {
        let ctx_guard = ctx.lock().await;
        let pool = &ctx_guard.state.db;

        // Normal items: fetch per item_id
        let mut items = Vec::new();
        for item_id in &changed_item_ids {
            if let Some(item) = items::get_item(pool, user_id, *item_id).await? {
                items.push(item);
            }
        }

        let mut power_items = Vec::new();
        for item_id in &changed_power_item_ids {
            let rows = items::get_power_item(pool, user_id, *item_id).await?;
            power_items.extend(rows);
        }

        let mut insight_items = Vec::new();
        for item_id in &changed_insight_item_ids {
            let rows = items::get_insight_item(pool, user_id, *item_id).await?;
            insight_items.extend(rows);
        }

        (items, power_items, insight_items)
    };

    if !items_list.is_empty() || !power_items_list.is_empty() || !insight_items_list.is_empty() {
        let mut ctx_guard = ctx.lock().await;
        let push = ItemChangePush {
            items: items_list.into_iter().map(Into::into).collect(),
            power_items: power_items_list.into_iter().map(Into::into).collect(),
            insight_items: insight_items_list.into_iter().map(Into::into).collect(),
        };

        ctx_guard
            .send_push(CmdId::ItemChangePushCmd, push.clone())
            .await?;

        tracing::info!(
            "Sent ItemChangePush: {} items, {} power items, {} insight items",
            push.items.len(),
            push.power_items.len(),
            push.insight_items.len()
        );
    }

    Ok(())
}

pub async fn send_currency_change_push(
    ctx: Arc<Mutex<ConnectionContext>>,
    user_id: i64,
    changed_currencies: Vec<(i32, i32)>,
) -> Result<(), AppError> {
    if changed_currencies.is_empty() {
        return Ok(());
    }

    let mut totals: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
    for (currency_id, amount) in &changed_currencies {
        *totals.entry(*currency_id).or_insert(0) += amount;
    }

    let currencies_list = {
        let ctx_guard = ctx.lock().await;
        let pool = &ctx_guard.state.db;

        let currency_ids: Vec<i32> = totals.keys().copied().collect();
        let mut currencies = Vec::new();
        for currency_id in currency_ids {
            if let Some(currency) = currencies::get_currency(pool, user_id, currency_id).await? {
                currencies.push(currency);
            }
        }

        currencies
    };

    if !currencies_list.is_empty() {
        let mut ctx_guard = ctx.lock().await;

        let mut details: Vec<String> = totals
            .iter()
            .map(|(id, total)| format!("{}x{}", id, total))
            .collect();
        details.sort();

        let push = CurrencyChangePush {
            change_currency: currencies_list
                .clone()
                .into_iter()
                .map(Into::into)
                .collect(),
        };

        ctx_guard
            .send_push(CmdId::CurrencyChangePushCmd, push)
            .await?;

        tracing::info!(
            "Sent CurrencyChangePush to user {}: {} currencies [{}] (added: {})",
            user_id,
            totals.len(),
            currencies_list
                .iter()
                .map(|c| format!("{}x{}", c.currency_id, c.quantity))
                .collect::<Vec<_>>()
                .join(", "),
            details.join(", ")
        );
    }

    Ok(())
}

/// Send material change push (reward notification popup)
/// Use raw tuples: (material_type, material_id, quantity)
pub async fn send_material_change_push(
    ctx: Arc<Mutex<ConnectionContext>>,
    rewards: Vec<(u32, u32, i32)>, // (material_type, material_id, quantity)
    get_approach: Option<u32>,     // Source of reward (25 = activity, etc.)
) -> Result<(), AppError> {
    if rewards.is_empty() {
        return Ok(());
    }

    let mut ctx_guard = ctx.lock().await;

    let push = MaterialChangePush {
        data_list: rewards
            .into_iter()
            .map(|(material_type, material_id, quantity)| MaterialData {
                materil_type: Some(material_type),
                materil_id: Some(material_id),
                quantity: Some(quantity),
            })
            .collect(),
        get_approach,
    };

    ctx_guard
        .send_push(CmdId::MaterialChangePushCmd, push.clone())
        .await?;

    tracing::info!(
        "Sent MaterialChangePush with {} materials",
        push.data_list.len()
    );

    Ok(())
}

pub async fn send_end_dungeon_push(
    ctx: Arc<Mutex<ConnectionContext>>,
    chapter_id: i32,
    episode_id: i32,
    normal_bonus: Vec<(u32, u32, i32)>,
) -> Result<(), AppError> {
    let normal_bonus = normal_bonus
        .into_iter()
        .map(|(t, id, q)| MaterialData {
            materil_type: Some(t),
            materil_id: Some(id),
            quantity: Some(q),
        })
        .collect();

    let push = EndDungeonPush {
        chapter_id: Some(chapter_id),
        episode_id: Some(episode_id),

        player_exp: Some(0),
        star: Some(2),

        first_bonus: vec![],
        normal_bonus,
        advenced_bonus: vec![],
        addition_bonus: vec![],
        time_first_bonus: vec![],
        drop_bonus: vec![],

        update_dungeon_record: Some(false),
        can_update_dungeon_record: Some(false),
        old_record_round: Some(0),
        new_record_round: Some(0),
        first_pass: Some(false),

        extra_str: Some(String::new()),
        assist_user_id: Some(0),
        assist_nickname: Some(String::new()),
        total_round: Some(0),
    };

    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_push(CmdId::DungeonEndDungeonPushCmd, push)
        .await?;

    Ok(())
}

pub async fn send_dungeon_update_push(
    ctx: Arc<Mutex<ConnectionContext>>,
    chapter_id: i32,
    episode_id: i32,
    star: i32,
    challenge_count: i32,
    has_record: bool,
    chapter_type: i32,        // e.g., 6 for episode chapter
    chapter_today_pass: i32,  // Today's completions for this chapter type
    chapter_today_total: i32, // Today's total attempts for this chapter type
) -> Result<(), AppError> {
    let dungeon_info = sonettobuf::UserDungeon {
        chapter_id: Some(chapter_id),
        episode_id: Some(episode_id),
        star: Some(star),
        challenge_count: Some(challenge_count),
        has_record: Some(has_record),
        left_return_all_num: Some(0),
        today_pass_num: Some(2),  // Episode-specific today count
        today_total_num: Some(2), // Episode-specific today total
    };

    let chapter_type_nums = vec![sonettobuf::UserChapterTypeNum {
        chapter_type: Some(chapter_type),
        today_pass_num: Some(chapter_today_pass),
        today_total_num: Some(chapter_today_total),
    }];

    let push = sonettobuf::DungeonUpdatePush {
        dungeon_info: Some(dungeon_info),
        chapter_type_nums,
    };

    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_push(CmdId::DungeonUpdatePushCmd, push)
        .await?;

    Ok(())
}

pub async fn send_equip_update_push(
    ctx: Arc<Mutex<ConnectionContext>>,
    user_id: i64,
    equip_ids: Vec<i32>,
) -> Result<(), AppError> {
    if equip_ids.is_empty() {
        return Ok(());
    }

    let equips = {
        let ctx_guard = ctx.lock().await;
        let pool = &ctx_guard.state.db;
        let mut all_equips = Vec::new();
        for equip_id in equip_ids {
            let equips: Vec<database::models::game::equipment::Equipment> = sqlx::query_as(
                "SELECT uid, user_id, equip_id, level, exp, break_lv, count, is_lock, refine_lv, created_at, updated_at
                 FROM equipment
                 WHERE user_id = ? AND equip_id = ? AND count > 0
                 ORDER BY uid"
            )
            .bind(user_id)
            .bind(equip_id)
            .fetch_all(pool)
            .await?;
            all_equips.extend(equips);
        }
        all_equips
    };

    let push = sonettobuf::EquipUpdatePush {
        equips: equips
            .into_iter()
            .map(|e| sonettobuf::Equip {
                equip_id: Some(e.equip_id),
                uid: Some(e.uid),
                level: Some(e.level),
                exp: Some(e.exp),
                break_lv: Some(e.break_lv),
                count: Some(e.count),
                is_lock: Some(e.is_lock),
                refine_lv: Some(e.refine_lv),
            })
            .collect(),
    };

    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_push(CmdId::EquipUpdatePushCmd, push.clone())
        .await?;

    tracing::info!(
        "Sent EquipUpdatePush to user {}: {} equipment items",
        user_id,
        push.equips.len()
    );

    Ok(())
}

pub async fn send_equip_update_push_by_uid(
    ctx: Arc<Mutex<ConnectionContext>>,
    user_id: i64,
    uids: &[i64],
) -> Result<(), AppError> {
    if uids.is_empty() {
        return Ok(());
    }

    let equips = {
        let ctx_guard = ctx.lock().await;
        let pool = &ctx_guard.state.db;

        let placeholders = std::iter::repeat("?")
            .take(uids.len())
            .collect::<Vec<_>>()
            .join(",");

        let sql = format!(
            "SELECT uid, user_id, equip_id, level, exp, break_lv, count, is_lock, refine_lv, created_at, updated_at
             FROM equipment
             WHERE user_id = ? AND uid IN ({}) AND count > 0",
            placeholders
        );

        let mut q =
            sqlx::query_as::<_, database::models::game::equipment::Equipment>(&sql).bind(user_id);

        for uid in uids {
            q = q.bind(uid);
        }

        q.fetch_all(pool).await?
    };

    let push = sonettobuf::EquipUpdatePush {
        equips: equips
            .into_iter()
            .map(|e| sonettobuf::Equip {
                uid: Some(e.uid),
                equip_id: Some(e.equip_id),
                level: Some(e.level),
                exp: Some(e.exp),
                break_lv: Some(e.break_lv),
                count: Some(e.count),
                is_lock: Some(e.is_lock),
                refine_lv: Some(e.refine_lv),
            })
            .collect(),
    };

    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_push(CmdId::EquipUpdatePushCmd, push.clone())
        .await?;

    tracing::info!(
        "Sent EquipUpdatePush to user {}: {} equipment items",
        user_id,
        push.equips.len()
    );

    Ok(())
}
