use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::charges;
use prost::Message;
#[allow(unused_imports)]
use sonettobuf::{
    CmdId, GainSpecialBlockPush, GetChargeInfoReply, GetMonthCardInfoReply, MaterialChangePush,
    MonthCardInfo, ReadChargeNewReply, ReadChargeNewRequest, UpdateRedDotPush,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_charge_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (charge_infos, sandbox) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        let infos = charges::get_charge_infos(&conn.state.db, player_id).await?;
        let sandbox = charges::get_sandbox_settings(&conn.state.db, player_id).await?;

        (infos, sandbox)
    };

    let reply = GetChargeInfoReply {
        infos: charge_infos.into_iter().map(Into::into).collect(),
        sandbox_enable: Some(sandbox.sandbox_enable),
        sandbox_balance: Some(sandbox.sandbox_balance),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetChargeInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_get_charge_push_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    {
        let mut conn = ctx.lock().await;
        conn.send_empty_reply(CmdId::GetChargePushInfoCmd, Vec::new(), 0, req.up_tag)
            .await?;
    }

    Ok(())
}

pub async fn on_get_month_card_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };

    let current_time = common::time::ServerTime::now_ms();
    let server_day = common::time::ServerTime::server_day(current_time);

    let active_cards: Vec<(i32, i64)> = sqlx::query_as(
        "SELECT card_id, end_time
         FROM user_month_card_history
         WHERE user_id = ? AND end_time > ?
         ORDER BY card_id",
    )
    .bind(player_id)
    .bind(current_time / 1000)
    .fetch_all(&pool)
    .await?;

    let claimed_today: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM user_month_card_days
         WHERE user_id = ? AND server_day = ?",
    )
    .bind(player_id)
    .bind(server_day)
    .fetch_optional(&pool)
    .await?;

    let already_claimed = claimed_today.is_some();

    let card_infos: Vec<MonthCardInfo> = active_cards
        .iter()
        .map(|(card_id, end_time)| MonthCardInfo {
            id: Some(*card_id),
            expire_time: Some(*end_time as i32),
            has_get_bonus: Some(already_claimed),
        })
        .collect();

    let reply = GetMonthCardInfoReply { infos: card_infos };
    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetMonthCardInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_read_charge_new(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = ReadChargeNewRequest::decode(&req.data[..])?;

    tracing::info!("Received ReadChargeNewRequest: {:?}", request);

    let data = ReadChargeNewReply {
        goods_ids: request.goods_ids,
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::ReadChargeNewCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
