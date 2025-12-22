use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::simple_property;
use sonettobuf::{CmdId, GetSimplePropertyReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_simple_property(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let properties = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;

        simple_property::get_simple_properties(&ctx_guard.state.db, player_id).await?
    };

    let reply = GetSimplePropertyReply {
        simple_properties: properties.into_iter().map(Into::into).collect(),
    };

    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_reply(CmdId::GetSimplePropertyCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
