use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;

#[allow(unused_imports)]
use sonettobuf::{CmdId, GainSpecialBlockPush, GetMonthCardInfoReply, MonthCardInfo};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_month_card_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        (
            ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?,
            ctx_guard.state.db.clone(),
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
    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_reply(CmdId::GetMonthCardInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
