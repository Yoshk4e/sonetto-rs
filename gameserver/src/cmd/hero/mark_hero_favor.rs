use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, MarkHeroFavorReply, MarkHeroFavorRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_mark_hero_favor(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = MarkHeroFavorRequest::decode(&req.data[..])?;
    tracing::info!("Received MarkHeroFavorRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let is_favor = request.is_favor.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(player_id, pool);
    hero.set_favor(hero_id, is_favor).await?;

    tracing::info!(
        "User {} set hero {} favorite status to {}",
        player_id,
        hero_id,
        is_favor
    );

    let data = MarkHeroFavorReply {
        hero_id: Some(hero_id),
        is_favor: Some(is_favor),
    };

    {
        let updated_hero = hero.get(hero_id).await?;

        let hero_proto: sonettobuf::HeroInfo = updated_hero.into();
        let push = HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };

        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(CmdId::HeroHeroUpdatePushCmd, push)
            .await?;

        tracing::info!(
            "Sent HeroUpdatePush for hero {} with favor {}",
            hero_id,
            is_favor
        );

        ctx_guard
            .send_reply(CmdId::MarkHeroFavorCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
