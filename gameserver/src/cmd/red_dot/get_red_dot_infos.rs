#![allow(unused_imports)]
use crate::packet::ClientPacket;
use crate::send_push;
use crate::state::ConnectionContext;
use crate::{data_loader::GameDataLoader, error::AppError};
use prost::Message;
use sonettobuf::{CmdId, GetRedDotInfosReply, GetRedDotInfosRequest, SimplePropertyPush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_red_dot_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = GetRedDotInfosRequest::decode(&req.data[..])?;
    let mut define_ids = request.ids;
    tracing::info!("Requested define_ids: {:?}", define_ids);

    // If ID 13 is requested, expand to include related IDs
    const EXPANSION_IDS: [i32; 3] = [1042, 1013, 1902];
    if define_ids.contains(&13) {
        tracing::info!("ID 13 detected, expanding to include: {:?}", EXPANSION_IDS);
        for id in EXPANSION_IDS {
            if !define_ids.contains(&id) {
                define_ids.push(id);
            }
        }
        tracing::info!("Expanded define_ids: {:?}", define_ids);
    }

    let response: GetRedDotInfosReply =
        GameDataLoader::load_struct("red_dot/red_dot_infos.json")
            .map_err(|e| AppError::Custom(format!("Failed to load: {}", e)))?;

    let reply = if define_ids.is_empty() {
        response
    } else {
        GetRedDotInfosReply {
            red_dot_infos: response
                .red_dot_infos
                .into_iter()
                .filter(|info| define_ids.contains(&info.define_id))
                .collect(),
        }
    };

    tracing::info!("Returning {} red dot infos", reply.red_dot_infos.len());

    /*const TRIGGER_IDS: [i32; 3] = [1042, 1013, 1902];
    let should_send_push = TRIGGER_IDS.iter().all(|id| define_ids.contains(id));

    if should_send_push {
        tracing::info!("All trigger IDs detected, sending property push");
        send_push!(
            ctx,
            CmdId::SimplePropertyPushCmd,
            SimplePropertyPush,
            "property/property_push_1.json"
        );
    }*/

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::GetRedDotInfosCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
