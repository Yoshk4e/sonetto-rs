use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::{error::AppError, utils::push::send_red_dot_push};
use database::db::user::account::get_user_token;

use sonettobuf::{CmdId, EndActivityPush, SummonQueryTokenReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_summon_query_token(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        (player_id, ctx_guard.state.db.clone())
    };

    let token = get_user_token(&pool, player_id).await?;

    {
        let mut ctx_guard = ctx.lock().await;
        let push = EndActivityPush { id: Some(12716) };
        ctx_guard.send_push(CmdId::EndActivityPushCmd, push).await?;
    }

    send_red_dot_push(Arc::clone(&ctx), player_id, Some(vec![1908])).await?;

    tracing::info!("before send_reply");

    {
        let mut ctx_guard = ctx.lock().await;
        let reply = SummonQueryTokenReply {
            token: Some(token.token),
        };

        ctx_guard
            .send_reply(CmdId::SummonQueryTokenCmd, reply, 0, req.up_tag)
            .await?;
    }

    tracing::info!("after send_reply");

    Ok(())
}
