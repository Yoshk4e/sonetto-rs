use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use chrono::Datelike;
use common::time::ServerTime;
use database::db::game::sign_in::{self, process_manual_sign_in};
use sonettobuf::{CmdId, SignInReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_sign_in(
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

    let now = ServerTime::now_ms();
    let adjusted = ServerTime::adjusted_datetime(now);
    let day_of_month = adjusted.day() as i32;

    let was_new_sign_in = process_manual_sign_in(&pool, player_id).await?;

    let day = if was_new_sign_in {
        Some(day_of_month)
    } else {
        None
    };

    let birthday_heroes = sign_in::get_birthday_heroes_today(&pool, player_id).await?;

    let data = SignInReply {
        day,
        birthday_hero_ids: birthday_heroes,
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::SignInCmd, data, 0, req.up_tag)
        .await?;

    Ok(())
}
