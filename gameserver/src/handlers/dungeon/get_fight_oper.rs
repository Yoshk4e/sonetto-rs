use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::battle::load_battle_replay;
use prost::Message;
use sonettobuf::{CmdId, GetFightOperReply, GetFightOperRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_fight_oper(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let _ = GetFightOperRequest::decode(&req.data[..])?;

    let (player_id, pool, episode_id, is_replay) = {
        let conn = ctx.lock().await;
        let battle = conn
            .active_battle
            .as_ref()
            .ok_or(AppError::InvalidRequest)?;

        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
            battle.replay_episode_id.unwrap_or_default(),
            battle.is_replay.unwrap_or(false),
        )
    };

    let oper_records = if is_replay {
        load_battle_replay(&pool, player_id, episode_id).await?
    } else {
        vec![]
    };

    let reply = GetFightOperReply { oper_records };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetFightOperCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
