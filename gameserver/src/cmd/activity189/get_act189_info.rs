use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::utils::common::send_message;
use sonettobuf::{CmdId, GetAct189InfoReply};
// use sonettobuf::GetAct189InfoRequest;
use tokio::net::TcpStream;

#[allow(dead_code)]
pub async fn on_get_act189_info(
    cmd_id: CmdId,
    socket: &mut TcpStream,
    _req: ClientPacket,
) -> Result<(), AppError> {
    let data = GetAct189InfoReply {
        activity_id: None,
        has_get_once_bonus: None,
    };

    send_message(socket, cmd_id, data, 0).await?;
    Ok(())
}
