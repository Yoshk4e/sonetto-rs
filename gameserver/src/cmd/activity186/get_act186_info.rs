use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::utils::common::send_message;
use sonettobuf::{CmdId, GetAct186InfoReply};
// use sonettobuf::{Act186Info, Act186TaskInfo, Act186LikeInfo, Act186GameInfo};
// use sonettobuf::GetAct186InfoRequest;
use tokio::net::TcpStream;

#[allow(dead_code)]
pub async fn on_get_act186_info(
    cmd_id: CmdId,
    socket: &mut TcpStream,
    _req: ClientPacket,
) -> Result<(), AppError> {
    let data = GetAct186InfoReply {
        activity_id: None,
        info: None,
        task_infos: Vec::new(),
        like_infos: Vec::new(),
        game_infos: Vec::new(),
    };

    send_message(socket, cmd_id, data, 0).await?;
    Ok(())
}
