use crate::error::AppError;
use crate::network::packet::ClientPacket;

use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetAssistBonusReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_assist_bonus(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let data = GetAssistBonusReply {
        assist_bonus: Some(0),
        has_receive_assist_bonus: Some(0),
    };

    let should_push = {
        let mut conn = ctx.lock().await;
        conn.check_and_mark_state_pushes().await?
    };

    if should_push {
        tracing::info!("Sending state pushes from GetAssistBonus");
    } else {
        tracing::warn!("No state pushes from GetAssistBonus");
    }

    {
        let mut conn = ctx.lock().await;

        conn.send_reply(CmdId::GetAssistBonusCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
