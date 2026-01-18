use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::push;
use crate::{error::AppError, util::inventory::add_currencies};

use prost::Message;
use sonettobuf::{CmdId, SignInAddupReply, SignInAddupRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_sign_in_addup(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SignInAddupRequest::decode(&req.data[..])?;
    let day = request.id.ok_or(AppError::InvalidRequest)?;

    tracing::info!("Sign-in addup claim request: day={}", day);

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };

    let addup_days: i32 =
        sqlx::query_scalar("SELECT addup_sign_in_day FROM user_sign_in_info WHERE user_id = ?")
            .bind(player_id)
            .fetch_optional(&pool)
            .await?
            .unwrap_or(0);

    let reward_amount = match day {
        7 => 30,
        15 => 60,
        25 => 90,
        _ => {
            tracing::info!("Invalid sign-in milestone day: {}, returning success", day);
            let reply = SignInAddupReply { id: Some(day) };
            let mut conn = ctx.lock().await;
            conn.send_reply(CmdId::SignInAddupCmd, reply, 0, req.up_tag)
                .await?;
            return Ok(());
        }
    };

    if addup_days < day {
        tracing::info!(
            "User {} tried to claim day {} bonus but only has {} sign-in days, returning success",
            player_id,
            day,
            addup_days
        );
        let reply = SignInAddupReply { id: Some(day) };
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::SignInAddupCmd, reply, 0, req.up_tag)
            .await?;
        return Ok(());
    }

    let already_claimed: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM user_sign_in_addup_bonus WHERE user_id = ? AND bonus_id = ?",
    )
    .bind(player_id)
    .bind(day)
    .fetch_optional(&pool)
    .await?;

    if already_claimed.is_some() {
        tracing::info!(
            "User {} already claimed day {} bonus, returning success",
            player_id,
            day
        );
        let reply = SignInAddupReply { id: Some(day) };
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::SignInAddupCmd, reply, 0, req.up_tag)
            .await?;
        return Ok(());
    }

    sqlx::query("INSERT INTO user_sign_in_addup_bonus (user_id, bonus_id) VALUES (?, ?)")
        .bind(player_id)
        .bind(day)
        .execute(&pool)
        .await?;

    tracing::info!(
        "User {} claimed sign-in day {} bonus - granting {} currency 11",
        player_id,
        day,
        reward_amount
    );

    let currencies = vec![(11, reward_amount)];
    let ids = add_currencies(&pool, player_id, &currencies).await?;

    push::send_currency_change_push(ctx.clone(), player_id, ids).await?;

    let material_changes = vec![(2, 11u32, reward_amount)];
    push::send_material_change_push(ctx.clone(), material_changes, Some(14)).await?;

    let reply = SignInAddupReply { id: Some(day) };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::SignInAddupCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
