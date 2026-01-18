use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;

use prost::Message;
use sonettobuf::{CmdId, SignInTotalRewardAllReply, SignInTotalRewardAllRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_sign_in_total_reward_all(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let _ = SignInTotalRewardAllRequest::decode(&req.data[..]);

    let reply = SignInTotalRewardAllReply { mark: Some(1022) };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::SignInTotalRewardAllCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
