use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use sonettobuf::CmdId;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_reconnect_fight(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_empty_reply(CmdId::ReconnectFightCmd, Vec::new(), 0, req.up_tag)
        .await?;
    Ok(())
}
