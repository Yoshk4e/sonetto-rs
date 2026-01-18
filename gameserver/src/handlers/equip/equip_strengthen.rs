use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::push;
use database::models::game::equipment::UserEquipmentModel;
use prost::Message;
use sonettobuf::{CmdId, EquipStrengthenReply, EquipStrengthenRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_equip_strengthen(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = EquipStrengthenRequest::decode(&req.data[..])?;

    let target_uid = request.target_uid.ok_or(AppError::InvalidRequest)?;
    let eat_equips = request.eat_equips.clone();

    if eat_equips.is_empty() {
        return Err(AppError::InvalidRequest);
    }

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };

    let equip = UserEquipmentModel::new(player_id, pool);

    let consume_items: Vec<(i64, i32)> = eat_equips
        .iter()
        .filter_map(|e| {
            let uid = e.eat_uid?;
            let count = e.count.unwrap_or(1);
            Some((uid, count))
        })
        .collect();

    let (total_exp, affected_equip_ids) = equip.strengthen_equip(target_uid, consume_items).await?;

    if total_exp == 0 {
        tracing::info!(
            "No valid equipment to consume for user {} target={}",
            player_id,
            target_uid
        );

        let reply = EquipStrengthenReply {
            target_uid: request.target_uid,
            eat_equips: request.eat_equips,
        };

        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::EquipStrengthenCmd, reply, 0, req.up_tag)
            .await?;

        return Ok(());
    }

    push::send_equip_update_push(ctx.clone(), player_id, affected_equip_ids).await?;

    let reply = EquipStrengthenReply {
        target_uid: request.target_uid,
        eat_equips: request.eat_equips,
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::EquipStrengthenCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
