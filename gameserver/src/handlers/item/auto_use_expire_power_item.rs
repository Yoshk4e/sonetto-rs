use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::push;
use crate::{error::AppError, handlers::item::util::can_claim_month_card};
use prost::Message;
use sonettobuf::{AutoUseExpirePowerItemReply, AutoUseExpirePowerItemRequest, CmdId};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_auto_use_expire_power_item(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = AutoUseExpirePowerItemRequest::decode(&req.data[..])?;
    tracing::info!("Received AutoUseExpirePowerItemRequest: {:?}", request);

    let (user_id, used_any) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let now = common::time::ServerTime::now_ms();
        let now_seconds = now / 1000;

        let expired_items: Vec<(i64, i32, i32, i64)> = sqlx::query_as(
            "SELECT uid, item_id, quantity, expire_time
             FROM power_items
             WHERE user_id = ? AND expire_time > 0 AND expire_time < ?",
        )
        .bind(player_id)
        .bind(now_seconds)
        .fetch_all(pool)
        .await?;

        if expired_items.is_empty() {
            tracing::info!("User {} has no expired power items", player_id);
            (player_id, false)
        } else {
            let game_data = data::exceldb::get();
            let mut total_stamina = 0;

            for (uid, item_id, quantity, expire_time) in &expired_items {
                if let Some(power_item) = game_data.power_item.iter().find(|p| p.id == *item_id) {
                    let stamina_gain = power_item.effect * quantity;
                    total_stamina += stamina_gain;

                    tracing::info!(
                        "Auto-using expired power item {} (uid: {}, qty: {}, effect: {}, expired at: {})",
                        item_id,
                        uid,
                        quantity,
                        power_item.effect,
                        expire_time
                    );

                    sqlx::query("DELETE FROM power_items WHERE uid = ? AND user_id = ?")
                        .bind(uid)
                        .bind(player_id)
                        .execute(pool)
                        .await?;
                }
            }

            if total_stamina > 0 {
                let current_stamina: i32 = sqlx::query_scalar(
                    "SELECT quantity FROM currencies WHERE user_id = ? AND currency_id = 4",
                )
                .bind(player_id)
                .fetch_optional(pool)
                .await?
                .unwrap_or(0);

                let new_stamina = current_stamina + total_stamina;

                sqlx::query(
                    "INSERT INTO currencies (user_id, currency_id, quantity, last_recover_time, expired_time)
                     VALUES (?, 4, ?, ?, 0)
                     ON CONFLICT(user_id, currency_id)
                     DO UPDATE SET quantity = ?"
                )
                .bind(player_id)
                .bind(new_stamina)
                .bind(now)
                .bind(new_stamina)
                .execute(pool)
                .await?;

                tracing::info!(
                    "User {} auto-converted {} expired power items into {} stamina (total: {})",
                    player_id,
                    expired_items.len(),
                    total_stamina,
                    new_stamina
                );
            }

            (player_id, true)
        }
    };

    let data = AutoUseExpirePowerItemReply {
        used: Some(used_any),
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::AutoUseExpirePowerItemCmd, data, 0, req.up_tag)
            .await?;
    }

    if used_any {
        push::send_currency_change_push(ctx.clone(), user_id, vec![(4, 0)]).await?;
    }

    can_claim_month_card(ctx.clone(), user_id).await?;

    let should_save = {
        let mut conn = ctx.lock().await;
        if let Some(ps) = conn.player_state.as_mut() {
            if !ps.initial_login_complete {
                tracing::info!("Completing initial login for player {}", ps.player_id);

                ps.last_state_push_sent_timestamp = None;
                ps.last_activity_push_sent_timestamp = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if should_save {
        let conn = ctx.lock().await;
        conn.save_current_player_state().await?;
    }

    Ok(())
}
