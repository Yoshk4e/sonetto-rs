use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::summon::update_sp_pool_up_heroes;
use prost::Message;

use sonettobuf::{ChooseEnhancedPoolHeroReply, ChooseEnhancedPoolHeroRequest, CmdId};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_choose_enhanced_pool_hero(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = ChooseEnhancedPoolHeroRequest::decode(&req.data[..])?;
    let pool_id = request.pool_id.ok_or(AppError::InvalidRequest)?;
    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        (player_id, ctx_guard.state.db.clone())
    };

    update_sp_pool_up_heroes(&pool, player_id, pool_id, &[hero_id]).await?;

    {
        let mut ctx_guard = ctx.lock().await;
        let reply = ChooseEnhancedPoolHeroReply {
            pool_id: Some(pool_id),
            hero_id: Some(hero_id),
        };

        ctx_guard
            .send_reply(CmdId::ChooseEnhancedPoolHeroCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
