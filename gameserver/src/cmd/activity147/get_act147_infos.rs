use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::utils::common::send_message;
use sonettobuf::{CmdId, GetAct147InfosReply};
// use sonettobuf::Act147Goods;
// use sonettobuf::GetAct147InfosRequest;
use tokio::net::TcpStream;

#[allow(dead_code)]
pub async fn on_get_act147_infos(
    cmd_id: CmdId,
    socket: &mut TcpStream,
    _req: ClientPacket,
) -> Result<(), AppError> {
    let data = GetAct147InfosReply {
        activity_id: None,
        act147_goods: Vec::new(),
        total_remain_count: None,
    };

    send_message(socket, cmd_id, data, 0).await?;
    Ok(())
}
