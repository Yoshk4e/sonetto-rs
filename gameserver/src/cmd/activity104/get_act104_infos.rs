use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, Get104InfosReply};
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub async fn on_get_act104_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::Get104InfosCmd,
        Get104InfosReply,
        "activity104/104_info.json"
    );
    Ok(())
}
