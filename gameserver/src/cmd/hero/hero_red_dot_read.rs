use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroRedDotReadReply, HeroRedDotReadRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_red_dot_read(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroRedDotReadRequest::decode(&req.data[..])?;
    tracing::info!("Received HeroRedDotReadRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let red_dot = 6; // find out how to map these // 6 means no redot

    let (user_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let user_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (user_id, pool)
    };

    let hero = UserHeroModel::new(user_id, pool);
    hero.read_hero_red_dot(hero_id, red_dot).await?;

    let data = HeroRedDotReadReply {
        hero_id: Some(hero_id),
        red_dot: Some(red_dot),
    };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::HeroRedDotReadCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
