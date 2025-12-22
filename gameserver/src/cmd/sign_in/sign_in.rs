use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use chrono::Datelike;
use common::time::ServerTime;
use database::db::game::sign_in;
use sonettobuf::{CmdId, SignInReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_sign_in(
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

    let now = ServerTime::now_ms();
    let _server_day = ServerTime::server_day(now) as i32;

    // Display-only day of month
    let server_date = ServerTime::server_date();
    let day_of_month = server_date.day() as i32;

    // Check if already signed in today
    /*let already_signed: Option<i32> =
        sqlx::query_scalar("SELECT 1 FROM user_sign_in_days WHERE user_id = ? AND server_day = ?")
            .bind(player_id)
            .bind(server_day)
            .fetch_optional(&pool)
            .await?;

    if already_signed.is_none() {
        // Only process sign-in if not already done today
        sign_in::process_manual_sign_in(&pool, player_id).await?;

        tracing::info!(
            "User {} manually signed in for day {}",
            player_id,
            day_of_month
        );
    } else {
        tracing::info!("User {} already signed in today, skipping", player_id);
    }*/

    let birthday_heroes = sign_in::get_birthday_heroes_today(&pool, player_id).await?;

    let data = SignInReply {
        day: Some(day_of_month),
        birthday_hero_ids: birthday_heroes,
    };

    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_reply(CmdId::SignInCmd, data, 0, req.up_tag)
        .await?;

    if let Some(state) = ctx_guard.player_state_mut() {
        state.record_sign_in(now);
        ctx_guard.save_current_player_state().await?;
    }

    Ok(())
}
