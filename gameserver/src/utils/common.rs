use crate::error::AppError;
use crate::packet::ServerPacket;
use sonettobuf::{CmdId, prost::Message};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[allow(dead_code)]
pub async fn send_message<T: Message + Default>(
    _socket: &mut TcpStream,
    _cmd_id: CmdId,
    data: T,
    _result_code: i16,
) -> Result<(), AppError> {
    let _data = data.encode_to_vec();
    //send_raw_buffer(socket, cmd_id, data, result_code).await?;
    Ok(())
}

pub async fn send_raw_server_message(
    socket: &mut TcpStream,
    cmd_id: CmdId,
    payload: Vec<u8>,
    result_code: i16,
    up_tag: u8,
    down_tag: u8,
) -> Result<(), AppError> {
    let packet = ServerPacket {
        cmd_id: cmd_id as i16,
        result_code: result_code as u16,
        up_tag,
        down_tag,
        data: payload,
    };

    socket.write_all(&packet.encode()).await?;
    Ok(())
}

pub fn encode_message<T: prost::Message>(msg: &T) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();
    msg.encode(&mut buf)
        .map_err(|e| AppError::Custom(e.to_string()))?;
    Ok(buf)
}
