use crate::error::AppError;
use crate::handlers::system::util::*;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::push::send_red_dot_push;
use common::time::ServerTime;
use database::db::game::sign_in;
use sonettobuf::{CmdId, Mail, NewMailPush};
use sqlx::Row;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_login(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    tracing::info!("→ Starting login handler");
    let login = parse_login_request(&req.data)?;
    let user_id = extract_user_id(&login.account_id)?;
    tracing::info!("→ Login attempt user_id={}", user_id);

    let (stored_token, token_expires_at) = {
        let db = {
            let ctx = ctx.lock().await;
            ctx.state.db.clone()
        };
        let row = sqlx::query("SELECT token, token_expires_at FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&db)
            .await?
            .ok_or_else(|| AppError::Custom("User not found".into()))?;
        (
            row.try_get::<String, _>("token")?,
            row.try_get::<Option<i64>, _>("token_expires_at")?,
        )
    };

    if stored_token != login.token {
        return login_error(&ctx, "Invalid token", req.up_tag).await;
    }

    let now = ServerTime::now_ms();
    if token_expires_at.is_some_and(|exp| now > exp) {
        return login_error(&ctx, "Token expired", req.up_tag).await;
    }

    tracing::info!("✓ Token validated for user_id={}", user_id);

    {
        let mut conn = ctx.lock().await;
        conn.load_player_state(user_id).await?;
    }

    {
        let db = {
            let ctx = ctx.lock().await;
            ctx.state.db.clone()
        };
        let (is_new_day, is_new_week, is_new_month) =
            sign_in::process_daily_login(&db, user_id).await?;
        if is_new_day {
            sign_in::reset_daily_counters(&db, user_id).await?;
        }
        if is_new_week {
            sign_in::reset_weekly_counters(&db, user_id).await?;
        }
        if is_new_month {
            sign_in::reset_monthly_counters(&db, user_id).await?;
        }
    }

    {
        let mut conn = ctx.lock().await;
        let now = ServerTime::now_ms();
        let today = ServerTime::server_day(now);

        conn.update_and_save_player_state(|state| {
            state.last_login_timestamp = Some(now);

            if state.is_new_server_day(now) {
                state.initial_login_complete = false;
                state.last_sign_in_day = today;
                state.last_daily_reset_time = Some(now);
                state.month_card_claimed = false;
                state.last_month_card_claim_timestamp = None;
            }

            if state.is_new_week(now) {
                state.last_weekly_reset_time = Some(now);
            }

            if state.is_new_month(now) {
                state.last_monthly_reset_time = Some(now);
            }

            state.mark_login_complete(now);
            state.last_sign_in_time = Some(now);
        })
        .await?;
    }

    send_red_dot_push(Arc::clone(&ctx), user_id, Some(vec![2218, 2220, 2221])).await?;
    send_red_dot_push(Arc::clone(&ctx), user_id, Some(vec![2240])).await?;
    send_red_dot_push(Arc::clone(&ctx), user_id, Some(vec![2230])).await?;
    send_critter_push(Arc::clone(&ctx), user_id).await?;

    {
        let conn = ctx.lock().await;
        let pool = &conn.state.db;
        let now = ServerTime::now_ms();

        let new_mails: Vec<(
            i64,
            i32,
            String,
            String,
            i32,
            i64,
            String,
            String,
            String,
            String,
            i64,
            i32,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT incr_id, mail_id, params, attachment, state, create_time,
                    sender, title, content, copy, expire_time, sender_type,
                    jump_title, jump
             FROM user_mails
             WHERE user_id = ? AND state = 0 AND (expire_time = 0 OR expire_time > ?)",
        )
        .bind(user_id)
        .bind(now)
        .fetch_all(pool)
        .await?;

        drop(conn);

        for (
            incr_id,
            mail_id,
            params,
            attachment,
            state,
            create_time,
            sender,
            title,
            content,
            copy,
            expire_time,
            sender_type,
            jump_title,
            jump,
        ) in new_mails.clone()
        {
            let mail = Mail {
                incr_id: Some(incr_id as u64),
                mail_id: Some(mail_id as u32),
                params: Some(params),
                attachment: Some(attachment),
                state: Some(state as u32),
                create_time: Some(create_time as u64),
                sender: Some(sender),
                title: Some(title),
                content: Some(content),
                copy: Some(copy),
                expire_time: Some(expire_time as u64),
                sender_type: Some(sender_type),
                jump_title: Some(jump_title),
                jump: Some(jump),
            };

            let mut conn = ctx.lock().await;
            conn.notify(CmdId::NewMailPushCmd, NewMailPush { mail: Some(mail) })
                .await?;
        }

        if !new_mails.is_empty() {
            tracing::info!(
                "Sent {} new mail notifications to user {}",
                new_mails.len(),
                user_id
            );
        }
    }

    {
        let mut conn = ctx.lock().await;
        let payload = build_login_reply(user_id);
        conn.send_raw_reply_fixed(CmdId::LoginRequestCmd, payload, 0, req.up_tag)
            .await?;
    }

    ConnectionContext::register(Arc::clone(&ctx)).await;
    tracing::info!("✓ Login successful for user_id={}", user_id);
    Ok(())
}
