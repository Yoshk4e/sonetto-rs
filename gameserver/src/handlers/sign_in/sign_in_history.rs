use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;

use database::db::game::sign_in;
use prost::Message;
use sonettobuf::{CmdId, SignInHistoryReply, SignInHistoryRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_sign_in_history(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SignInHistoryRequest::decode(&req.data[..]);

    let month = request.unwrap().month.ok_or(AppError::InvalidRequest)?;

    let (_, sign_in_days, _, month_card_days, _, birthday_heroes) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        sign_in::get_sign_in_info(&conn.state.db, player_id).await?
    };

    let reply = SignInHistoryReply {
        month: Some(month),
        has_sign_in_days: sign_in_days,
        has_month_card_days: month_card_days,
        birthday_hero_ids: birthday_heroes,
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::SignInHistoryCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
