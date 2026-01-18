use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::{error::AppError, state::send_end_fight_push};
use prost::Message;
use sonettobuf::{CmdId, EndDungeonReply, EndDungeonRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_dungeon_end_dungeon(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = EndDungeonRequest::decode(&req.data[..])?;

    let is_abort = request.is_abort.ok_or(AppError::InvalidRequest)?;

    tracing::info!("Dungeon ended with is_abort: {}", is_abort);

    let (fight_group, is_replay, battle_id) = {
        let conn = ctx.lock().await;
        let battle = conn
            .active_battle
            .as_ref()
            .ok_or(AppError::InvalidRequest)?;

        (
            battle.fight_group.clone(),
            battle.is_replay.unwrap_or(false),
            battle.fight_id.unwrap_or_default(),
        )
    };

    if is_abort {
        send_end_fight_push(
            ctx.clone(),
            battle_id,
            -1, // abort
            fight_group.clone().unwrap_or_default(),
            vec![],
            vec![],
            !is_replay,
        )
        .await?;
    }

    {
        let mut conn = ctx.lock().await;
        conn.active_battle = None;
    }

    let data = EndDungeonReply {};

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::DungeonEndDungeonCmd, data, 0, req.up_tag)
        .await?;

    Ok(())
}
