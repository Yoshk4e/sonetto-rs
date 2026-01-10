use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, DestinyStoneUseReply, DestinyStoneUseRequest, HeroUpdatePush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_destiny_stone_use(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = DestinyStoneUseRequest::decode(&req.data[..])?;
    tracing::info!("Received DestinyStoneUseRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let stone_id = request.stone_id.ok_or(AppError::InvalidRequest)?;

    let updated_hero = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let hero = UserHeroModel::new(player_id, pool.clone());
        let hero_data = hero.get(hero_id).await?;

        let hero_info: sonettobuf::HeroInfo = hero_data.into();

        hero.update_destiny_stone(hero_id, stone_id).await?;

        tracing::info!(
            "User {} equipped destiny stone {} on hero {}",
            player_id,
            stone_id,
            hero_id
        );
        hero_info
    };

    let data = DestinyStoneUseReply {
        hero_id: Some(hero_id),
        stone_id: Some(stone_id),
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
            .send_reply(CmdId::DestinyStoneUseCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
