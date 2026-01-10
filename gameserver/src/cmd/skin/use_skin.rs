use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, UseSkinReply, UseSkinRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_use_skin(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UseSkinRequest::decode(&req.data[..])?;
    tracing::info!("Received UseSkinRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let skin_id = request.skin_id.ok_or(AppError::InvalidRequest)?;

    let updated_hero = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let hero = UserHeroModel::new(player_id, pool.clone());
        let hero_data = hero.get(hero_id).await?;
        let hero_info: sonettobuf::HeroInfo = hero_data.into();

        hero.update_skin(hero_id, skin_id).await?;

        tracing::info!(
            "User {} equipped skin {} on hero {}",
            player_id,
            skin_id,
            hero_id
        );
        hero_info
    };

    let data = UseSkinReply {
        hero_id: Some(hero_id),
        skin_id: Some(skin_id),
    };

    {
        let mut ctx_guard = ctx.lock().await;

        let hero_proto: sonettobuf::HeroInfo = updated_hero.into();
        let push = HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };

        ctx_guard
            .send_push(CmdId::HeroHeroUpdatePushCmd, push)
            .await?;

        ctx_guard
            .send_reply(CmdId::UseSkinCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
