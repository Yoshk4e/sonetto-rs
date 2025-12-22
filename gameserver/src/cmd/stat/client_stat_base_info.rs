use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::user_stats;
use sonettobuf::CmdId;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_client_stat_base_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let stats = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;

        user_stats::get_user_stats(&ctx_guard.state.db, player_id)
            .await?
            .ok_or_else(|| AppError::Custom("User stats not found".to_string()))?
    };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(
                CmdId::StatInfoPushCmd,
                <sonettobuf::StatInfoPush>::from(stats),
            )
            .await?;
    }

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_empty_reply(CmdId::ClientStatBaseInfoCmd, Vec::new(), 0, req.up_tag)
            .await?;
    }

    Ok(())
}
