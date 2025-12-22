use crate::error::AppError;
use crate::state::ConnectionContext;
use database::db::game::{items, red_dots};
use sonettobuf::{
    CmdId, EndDungeonPush, ItemChangePush, MaterialChangePush, MaterialData, UpdateRedDotPush,
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
) -> Result<(), AppError> {
    if changed_item_ids.is_empty() {
        return Ok(());
    }

    let (items_list, power_items_list, insight_items_list) = {
        let ctx_guard = ctx.lock().await;
        let pool = &ctx_guard.state.db;

        let mut items = Vec::new();
        for item_id in &changed_item_ids {
            if let Some(item) = items::get_item(pool, user_id, *item_id).await? {
                items.push(item);
            }
        }

        let power_items = items::get_all_power_items(pool, user_id).await?;

        let insight_items = items::get_all_insight_items(pool, user_id).await?;

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
