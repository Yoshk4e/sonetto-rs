use crate::cmd::system::util::*;
use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::utils::push::send_red_dot_push;

use database::db::game::sign_in;
use sonettobuf::CmdId;

use sqlx::Row;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_login(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    tracing::info!("→ Starting login handler");

    let login = parse_login_request(&req.data)?;
    tracing::info!(
        "→ Parsed LoginRequest - Account: {}, Token: {}...",
        login.account_id,
        &login.token[..8.min(login.token.len())]
    );

    // Extract user_id from account_id format: "200_69420" -> 69420
    let user_id = match extract_user_id(&login.account_id) {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("Failed to parse account_id: {}", e);
            let mut ctx_guard = ctx.lock().await;
            let payload = build_login_error("Invalid account format");
            ctx_guard
                .send_raw_reply_fixed(CmdId::LoginRequestCmd, payload, 1, req.up_tag)
                .await?;
            return Err(e);
        }
    };

    tracing::info!("→ Extracted user_id: {}", user_id);

    // Validate token from database using user_id
    let user_result = {
        let ctx_guard = ctx.lock().await;
        sqlx::query("SELECT id, token, token_expires_at FROM users WHERE id = ?1")
            .bind(user_id)
            .fetch_optional(&ctx_guard.state.db)
            .await
    };

    let user_row = match user_result {
        Ok(Some(row)) => row,
        Ok(None) => {
            tracing::warn!("User not found: {}", user_id);
            let mut ctx_guard = ctx.lock().await;
            let payload = build_login_error("User not found");
            ctx_guard
                .send_raw_reply_fixed(CmdId::LoginRequestCmd, payload, 1, req.up_tag)
                .await?;
            return Err(AppError::Custom("User not found".to_string()));
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            let mut ctx_guard = ctx.lock().await;
            let payload = build_login_error("Database error");
            ctx_guard
                .send_raw_reply_fixed(CmdId::LoginRequestCmd, payload, 1, req.up_tag)
                .await?;
            return Err(AppError::Database(e));
        }
    };

    // Extract and validate token
    let stored_token: String = user_row.try_get("token")?;
    let token_expires_at: Option<i64> = user_row.try_get("token_expires_at").ok().flatten();

    // Validate token
    if stored_token != login.token {
        tracing::warn!("Invalid token for user {}", user_id);
        let mut ctx_guard = ctx.lock().await;
        let payload = build_login_error("Invalid token");
        ctx_guard
            .send_raw_reply_fixed(CmdId::LoginRequestCmd, payload, 1, req.up_tag)
            .await?;
        return Err(AppError::Custom("Invalid token".to_string()));
    }

    // Check token expiry
    let now = common::time::ServerTime::now_ms() as i64;
    if let Some(expires_at) = token_expires_at {
        if now > expires_at {
            tracing::warn!("Expired token for user {}", user_id);
            let mut ctx_guard = ctx.lock().await;
            let payload = build_login_error("Token expired");
            ctx_guard
                .send_raw_reply_fixed(CmdId::LoginRequestCmd, payload, 1, req.up_tag)
                .await?;
            return Err(AppError::Custom("Token expired".to_string()));
        }
    }

    tracing::info!("✓ Token validated for user_id: {}", user_id);

    send_red_dot_push(Arc::clone(&ctx), user_id, Some(vec![2218, 2220, 2221])).await?;

    send_red_dot_push(Arc::clone(&ctx), user_id, Some(vec![2240])).await?;

    send_red_dot_push(Arc::clone(&ctx), user_id, Some(vec![2230])).await?;

    send_critter_push(Arc::clone(&ctx), user_id).await?;

    // Load player state and register session
    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard.load_player_state(user_id).await?;

        let payload = build_login_reply(user_id);
        tracing::debug!("[LoginReply] payload: {:02X?}", payload);

        ctx_guard
            .send_raw_reply_fixed(CmdId::LoginRequestCmd, payload, 0, req.up_tag)
            .await?;
    }

    {
        let ctx_guard = ctx.lock().await;
        let pool = &ctx_guard.state.db;

        // Process daily login and check for resets
        let (is_new_day, is_new_week, _is_new_month) =
            sign_in::process_daily_login(pool, user_id).await?;

        if is_new_day {
            sign_in::reset_daily_counters(pool, user_id).await?;
        }

        if is_new_week {
            sign_in::reset_weekly_counters(pool, user_id).await?;
        }
    }

    ConnectionContext::register(Arc::clone(&ctx)).await;

    tracing::info!("✓ Login successful for user_id: {}", user_id);

    Ok(())
}
