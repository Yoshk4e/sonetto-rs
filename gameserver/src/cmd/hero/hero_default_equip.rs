use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroDefaultEquipReply, HeroDefaultEquipRequest, HeroUpdatePush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_default_equip(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroDefaultEquipRequest::decode(&req.data[..])?;
    tracing::info!("Received HeroDefaultEquipRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let equip_uid = request.default_equip_uid.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(player_id, pool);
    hero.update_equipped_gear(hero_id, equip_uid).await?;

    tracing::info!(
        "User {} equipped gear {} on hero {}",
        player_id,
        equip_uid,
        hero_id
    );

    let data = HeroDefaultEquipReply {
        hero_id: Some(hero_id),
        default_equip_uid: Some(equip_uid),
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
            "Sent HeroUpdatePush for hero {} with equip {}",
            hero_id,
            equip_uid
        );

        ctx_guard
            .send_reply(CmdId::HeroDefaultEquipCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
