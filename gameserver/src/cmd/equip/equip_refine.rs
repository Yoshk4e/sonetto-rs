use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::utils::push;
use database::models::game::equipment::{EquipmentModel, UserEquipmentModel};
use prost::Message;
use sonettobuf::{CmdId, EquipRefineReply, EquipRefineRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_equip_refine(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = EquipRefineRequest::decode(&req.data[..])?;

    let target_uid = request.target_uid.ok_or(AppError::InvalidRequest)?;
    let eat_uids = request.eat_uids.clone();

    if eat_uids.is_empty() {
        return Err(AppError::InvalidRequest);
    }

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        (
            ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?,
            ctx_guard.state.db.clone(),
        )
    };

    let equip = UserEquipmentModel::new(player_id, pool.clone());

    let mut target = equip.get_equip(target_uid).await?;

    let game_data = data::exceldb::get();

    let mut valid_consumed = 0;
    let mut equips_to_delete = Vec::new();

    for eat_uid in &eat_uids {
        let eat_equipment = match equip.get_equip(*eat_uid).await {
            Ok(equip) => equip,
            Err(e) => {
                tracing::warn!(
                    "Failed to get equipment uid={} for user {}: {:?}, skipping",
                    eat_uid,
                    player_id,
                    e
                );
                continue;
            }
        };

        if eat_equipment.is_lock {
            tracing::info!("Skipping locked equipment uid={} in refine", eat_uid);
            continue;
        }

        if *eat_uid == target_uid {
            continue;
        }

        match game_data.equip.get(eat_equipment.equip_id) {
            Some(data) => data,
            None => {
                tracing::warn!(
                    "Equipment data not found for equip_id={}, skipping",
                    eat_equipment.equip_id
                );
                continue;
            }
        };

        let is_valid = if eat_equipment.equip_id == target.equip_id {
            true
        } else if eat_equipment.equip_id == 1000 || eat_equipment.equip_id == 1001 {
            true
        } else {
            false
        };

        if !is_valid {
            tracing::info!(
                "Equipment {} cannot be used to refine equipment {}",
                eat_equipment.equip_id,
                target.equip_id
            );
            continue;
        }

        valid_consumed += 1;
        equips_to_delete.push((*eat_uid, eat_equipment.equip_id));
    }

    let mut affected_uids = Vec::with_capacity(1 + equips_to_delete.len());
    affected_uids.push(target_uid);

    for (eat_uid, _) in &equips_to_delete {
        affected_uids.push(*eat_uid);
    }

    target.refine_lv += valid_consumed;

    let max_refine_lv = 5;
    if target.refine_lv > max_refine_lv {
        target.refine_lv = max_refine_lv;
    }

    if equip.refine_level(target_uid, target.refine_lv).await? {
        tracing::info!(
            "User {} refined equipment uid={} to refine_lv {} (consumed {} items)",
            player_id,
            target_uid,
            target.refine_lv,
            valid_consumed
        );
    }

    let mut affected_equip_ids = vec![target.equip_id];
    for (_, equip_id) in &equips_to_delete {
        if !affected_equip_ids.contains(equip_id) {
            affected_equip_ids.push(*equip_id);
        }
    }

    for (eat_uid, _) in &equips_to_delete {
        equip.delete(*eat_uid).await?;
    }

    let delete_uids: Vec<i64> = equips_to_delete.iter().map(|(uid, _)| *uid).collect();
    if !delete_uids.is_empty() {
        let delete_push = sonettobuf::EquipDeletePush {
            uids: delete_uids.clone(),
        };
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(CmdId::EquipDeletePushCmd, delete_push)
            .await?;
    }

    push::send_equip_update_push(ctx.clone(), player_id, affected_equip_ids).await?;

    let reply = EquipRefineReply {
        target_uid: request.target_uid,
        eat_uids: request.eat_uids,
    };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::EquipRefineCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
