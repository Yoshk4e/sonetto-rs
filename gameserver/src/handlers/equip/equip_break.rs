use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::push;

use database::models::game::{
    currencies::UserCurrencyModel,
    equipment::{EquipmentModel, UserEquipmentModel},
    items::UserItemModel,
};

use prost::Message;
use sonettobuf::{CmdId, EquipBreakReply, EquipBreakRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_equip_break(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = EquipBreakRequest::decode(&req.data[..])?;

    let target_uid = request.target_uid.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };

    let equip = UserEquipmentModel::new(player_id, pool.clone());
    let item = UserItemModel::new(player_id, pool.clone());
    let currency = UserCurrencyModel::new(player_id, pool.clone());

    let mut target = equip.get_equip(target_uid).await?;

    let game_data = data::exceldb::get();
    let target_equip_data = game_data
        .equip
        .get(target.equip_id)
        .ok_or(AppError::InvalidRequest)?;

    let rare = target_equip_data.rare;

    let current_break = game_data
        .equip_break_cost
        .iter()
        .find(|e| e.rare == rare && e.break_level == target.break_lv)
        .ok_or(AppError::InvalidRequest)?;

    if target.level < current_break.level {
        tracing::info!(
            "User {} tried to break equipment uid={} at level {} but needs level {} for current break_lv {}",
            player_id,
            target_uid,
            target.level,
            current_break.level,
            target.break_lv
        );

        let reply = EquipBreakReply {};

        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::EquipBreakCmd, reply, 0, req.up_tag)
            .await?;

        return Ok(());
    }

    let next_break = game_data
        .equip_break_cost
        .iter()
        .find(|e| e.rare == rare && e.break_level == target.break_lv + 1);

    let Some(next_break) = next_break else {
        tracing::info!(
            "User {} tried to break equipment uid={} but already at max break level",
            player_id,
            target_uid
        );

        let reply = EquipBreakReply {};

        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::EquipBreakCmd, reply, 0, req.up_tag)
            .await?;

        return Ok(());
    };

    if next_break.score_cost > 0 {
        let success = currency.remove_currency(1, next_break.score_cost).await?;
        if !success {
            tracing::warn!(
                "User {} insufficient gold for break (needs {})",
                player_id,
                next_break.score_cost
            );

            let reply = EquipBreakReply {};

            let mut conn = ctx.lock().await;
            conn.send_reply(CmdId::EquipBreakCmd, reply, 0, req.up_tag)
                .await?;

            return Ok(());
        }
    }

    if !next_break.cost.is_empty() {
        let (items, _, _, _, _, _) = crate::state::parse_store_product(&next_break.cost);

        for (item_id, amount) in &items {
            let success = item.remove_item_quantity(*item_id, *amount).await?;
            if !success {
                tracing::warn!(
                    "User {} insufficient items for break (item_id={}, needs={})",
                    player_id,
                    item_id,
                    amount
                );

                let reply = EquipBreakReply {};

                let mut conn = ctx.lock().await;
                conn.send_reply(CmdId::EquipBreakCmd, reply, 0, req.up_tag)
                    .await?;

                return Ok(());
            }
        }
    }

    target.break_lv += 1;

    if equip.break_level(target_uid).await? {
        tracing::info!(
            "User {} broke through equipment uid={} to break_lv {} (cost {} sharp)",
            player_id,
            target_uid,
            target.break_lv,
            next_break.score_cost
        );
    }

    if next_break.score_cost > 0 {
        push::send_currency_change_push(ctx.clone(), player_id, vec![(1, 0)]).await?;
    }

    push::send_equip_update_push(ctx.clone(), player_id, vec![target.equip_id]).await?;

    let reply = EquipBreakReply {};

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::EquipBreakCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
