use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::utils::push;
use database::models::game::equipment::UserEquipmentModel;
use prost::Message;
use sonettobuf::{CmdId, EquipLockReply, EquipLockRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_equip_lock(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = EquipLockRequest::decode(&req.data[..])?;
    let target_uid = request.target_uid.unwrap_or(0);
    let lock = request.lock.unwrap_or(false);

    let (updated, user_id, equip_id) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let equip = UserEquipmentModel::new(player_id, pool.clone());

        let updated = equip.update_equipment_lock(target_uid, lock).await?;

        let equip_id = if updated {
            equip.get_equip(target_uid).await.ok().map(|e| e.equip_id)
        } else {
            None
        };

        (updated, player_id, equip_id)
    };

    if !updated {
        tracing::warn!("Equipment {} not found or not owned by user", target_uid);
        return Err(AppError::InvalidRequest);
    }

    let reply = EquipLockReply {
        target_uid: Some(target_uid),
        lock: Some(lock),
    };

    {
        let mut ctx_guard = ctx.lock().await;

        ctx_guard
            .send_reply(CmdId::EquipLockCmd, reply, 0, req.up_tag)
            .await?;
    }

    if let Some(equip_id) = equip_id {
        push::send_equip_update_push(ctx.clone(), user_id, vec![equip_id]).await?;
    }

    Ok(())
}
