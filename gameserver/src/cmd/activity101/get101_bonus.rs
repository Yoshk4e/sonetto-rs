use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::{error::AppError, utils::push};
use database::db::game::activity101;
use prost::Message;
use sonettobuf::{CmdId, Get101BonusReply, Get101BonusRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get101_bonus(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = Get101BonusRequest::decode(&req.data[..])?;

    let activity_id = request.activity_id.ok_or(AppError::InvalidRequest)?;
    let day_id = request.id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        (
            ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?,
            ctx_guard.state.db.clone(),
        )
    };

    let mut debug_info = String::new();
    {
        let ctx_guard = ctx.lock().await;

        if let Some(state) = &ctx_guard.player_state {
            // ALWAYS use server time for reset logic
            let now = common::time::ServerTime::now_ms();

            debug_info = format!(
                "DEBUG Get101Bonus:\n\
                 - last_daily_reward_time: {:?}\n\
                 - ServerTime::now_ms(): {}\n\
                 - server_day(now): {}\n\
                 - server_day(last): {:?}\n\
                 - is_new_day_for_rewards(server): {}",
                state.last_daily_reward_time,
                now,
                common::time::ServerTime::server_day(now),
                state
                    .last_daily_reward_time
                    .map(common::time::ServerTime::server_day),
                state.is_new_reward_day(now),
            );
        }
    }

    tracing::info!("{}", debug_info);

    // Check if already claimed
    let already_claimed: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM user_activity101_claims
         WHERE user_id = ? AND activity_id = ? AND day_id = ?",
    )
    .bind(player_id)
    .bind(activity_id)
    .bind(day_id)
    .fetch_optional(&pool)
    .await?;

    if already_claimed.is_some() {
        tracing::warn!(
            "User {} already claimed day {} for activity {}",
            player_id,
            day_id,
            activity_id
        );

        let reply = Get101BonusReply {
            activity_id: Some(activity_id),
            id: Some(day_id),
        };

        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::Get101BonusCmd, reply, 0, req.up_tag)
            .await?;
        return Ok(());
    }

    // Claim the reward
    activity101::claim_activity101_day(&pool, player_id, activity_id, day_id as i32).await?;

    let item_rewards = vec![(140001_u32, 1_i32)]; // (item_id, quantity)
    let currency_rewards = vec![];

    // Add items to inventory
    let mut changed_item_ids = Vec::new();
    for (item_id, quantity) in &item_rewards {
        database::db::game::items::add_item_quantity(&pool, player_id, *item_id as u32, *quantity)
            .await?;
        changed_item_ids.push(*item_id as u32);
    }

    // Add currencies
    let mut changed_currency_ids = Vec::new();
    for (currency_id, amount) in &currency_rewards {
        database::db::game::currencies::add_currency(&pool, player_id, *currency_id, *amount)
            .await?;
        changed_currency_ids.push(*currency_id);
    }

    tracing::info!(
        "User {} claimed day {} for activity {}: {} items, {} currencies",
        player_id,
        day_id,
        activity_id,
        changed_item_ids.len(),
        changed_currency_ids.len()
    );

    // Build material rewards for popup notification
    let material_rewards = vec![(1, 140001, 1)];

    // Send all pushes
    push::send_item_change_push(ctx.clone(), player_id, changed_item_ids).await?;
    push::send_red_dot_push(ctx.clone(), player_id, Some(vec![2240])).await?;
    push::send_material_change_push(ctx.clone(), material_rewards, Some(25)).await?; // 25 = activity source

    push::send_red_dot_push(ctx.clone(), player_id, Some(vec![1010])).await?;
    push::send_red_dot_push(ctx.clone(), player_id, Some(vec![30558, 30557])).await?;

    let reply = Get101BonusReply {
        activity_id: Some(activity_id),
        id: Some(day_id),
    };

    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_reply(CmdId::Get101BonusCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
