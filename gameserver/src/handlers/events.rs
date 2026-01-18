use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use crate::{
    error::AppError,
    util::{
        data_loader::GameDataLoader,
        inventory::{add_currencies, add_items},
        push,
    },
};
use database::db::game::activity101;
use prost::Message;
use sonettobuf::{
    Act101Info, Act160GetInfoReply, Act165GetInfoReply, CmdId, Get101BonusReply,
    Get101BonusRequest, Get101InfosReply, Get101InfosRequest, GetAct125InfosReply,
    GetAct125InfosRequest, GetAct208InfoReply, GetAct209InfoReply, GetActivityInfosReply,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_activity_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetActivityInfosCmd,
        GetActivityInfosReply,
        "activity/activity_infos.json"
    );
    Ok(())
}

pub async fn on_get101_bonus(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = Get101BonusRequest::decode(&req.data[..])?;

    let activity_id = request.activity_id.ok_or(AppError::InvalidRequest)?;
    let day_id = request.id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };

    let now = common::time::ServerTime::now_ms();

    {
        let conn = ctx.lock().await;

        if let Some(state) = &conn.player_state {
            tracing::debug!("Get101Bonus");
            tracing::debug!("last_daily_reward_time: {:?}", state.last_daily_reward_time);
            tracing::debug!("ServerTime: {:?}", now);
            tracing::debug!(
                "CurrServerDay: {:?}",
                common::time::ServerTime::server_day(now)
            );
            tracing::debug!(
                "LastServerDay: {:?}",
                state
                    .last_daily_reward_time
                    .map(common::time::ServerTime::server_day)
            );
            tracing::debug!("IsNewRewardDay: {:?}", state.is_new_reward_day(now));
        }
    }

    // Check if already claimed
    let claimed_at: Option<i64> = sqlx::query_scalar(
        "SELECT claimed_at
         FROM user_activity101_claims
         WHERE user_id = ? AND activity_id = ? AND day_id = ?",
    )
    .bind(player_id)
    .bind(activity_id)
    .bind(day_id)
    .fetch_optional(&pool)
    .await?
    .flatten();

    if claimed_at.is_some() {
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

        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::Get101BonusCmd, reply, 0, req.up_tag)
            .await?;
        return Ok(());
    }

    activity101::claim_activity101_day(&pool, player_id, activity_id, day_id as i32).await?;

    {
        let mut conn = ctx.lock().await;

        conn.update_and_save_player_state(|state| {
            state.mark_daily_reward_claimed(now);
        })
        .await?;
    }

    let item_rewards = vec![(140001_u32, 1_i32)]; // (item_id, quantity)
    let currency_rewards = vec![];

    let changed_item_ids = add_items(&pool, player_id, &item_rewards).await?;
    let changed_currency_ids = add_currencies(&pool, player_id, &currency_rewards).await?;

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
    push::send_item_change_push(ctx.clone(), player_id, changed_item_ids, vec![], vec![]).await?;
    push::send_red_dot_push(ctx.clone(), player_id, Some(vec![2240])).await?;
    push::send_material_change_push(ctx.clone(), material_rewards, Some(25)).await?; // 25 = activity source

    push::send_red_dot_push(ctx.clone(), player_id, Some(vec![1010])).await?;
    push::send_red_dot_push(ctx.clone(), player_id, Some(vec![30558, 30557])).await?;

    let reply = Get101BonusReply {
        activity_id: Some(activity_id),
        id: Some(day_id),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::Get101BonusCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_get101_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = Get101InfosRequest::decode(&req.data[..])?;
    let activity_id = request.activity_id.unwrap_or(13108);

    tracing::info!("Requested activity_id: {}", activity_id);

    let (infos, login_count, got_once_bonus) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        activity101::get_activity101_info(&conn.state.db, player_id, activity_id).await?
    };

    let reply = Get101InfosReply {
        infos: infos
            .into_iter()
            .map(|(id, state)| Act101Info {
                id: Some(id as u32),
                state: Some(state as u32),
            })
            .collect(),
        sp_infos: vec![],
        login_count: Some(login_count as u32),
        activity_id: Some(activity_id),
        got_once_bonus: Some(got_once_bonus),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::Get101InfosCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_get_act125_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = GetAct125InfosRequest::decode(&req.data[..])?;
    let activity_id = request.activity_id.unwrap_or(0);

    tracing::info!("Requested activity_id: {}", activity_id);

    let path = match activity_id {
        13116 => "activity125/activity125_infos_13116.json",
        13005 => "activity125/activity125_infos_13005.json",
        _ => {
            tracing::warn!("Unknown activity_id: {}, using default", activity_id);
            "activity125/activity125_infos_13116.json"
        }
    };

    let reply: GetAct125InfosReply = GameDataLoader::load_struct(path)
        .map_err(|e| AppError::Custom(format!("Failed to load: {}", e)))?;

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetAct125InfosCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_act160_get_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::Act160GetInfoCmd,
        Act160GetInfoReply,
        "activity160/get_info.json"
    );
    Ok(())
}

pub async fn on_act165_get_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::Act165GetInfoCmd,
        Act165GetInfoReply,
        "activity165/get_info.json"
    );
    Ok(())
}

pub async fn on_get_act208_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetAct208InfoCmd,
        GetAct208InfoReply,
        "activity208/get_info.json"
    );
    Ok(())
}

pub async fn on_get_act209_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetAct209InfoCmd,
        GetAct209InfoReply,
        "activity209/get_info.json"
    );
    Ok(())
}
