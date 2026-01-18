use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{ChoiceHero3123WeaponReply, ChoiceHero3123WeaponRequest, CmdId, HeroUpdatePush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_choice_hero_3123_weapon(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = ChoiceHero3123WeaponRequest::decode(&req.data[..])?;

    tracing::info!("Received ChoiceHero3123WeaponRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let main_id = request.main_id.ok_or(AppError::InvalidRequest)?;
    let sub_id = request.sub_id.ok_or(AppError::InvalidRequest)?;

    let special_equip = format!("{}#{}", main_id, sub_id);

    let data = ChoiceHero3123WeaponReply {
        hero_id: Some(hero_id),
        main_id: Some(main_id),
        sub_id: Some(sub_id),
    };

    let updated_hero = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let hero = UserHeroModel::new(player_id, pool.clone());

        let ezio = hero.get(hero_id).await?;

        let hero_info: sonettobuf::HeroInfo = ezio.into();

        hero.update_special_equipped_gear(hero_id, special_equip.clone())
            .await?;

        tracing::info!(
            "User {} equipped gear {} on hero {}",
            player_id,
            special_equip,
            hero_id
        );

        hero_info
    };

    {
        let mut conn = ctx.lock().await;

        let hero_proto: sonettobuf::HeroInfo = updated_hero;
        let push = HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };

        conn.notify(CmdId::HeroHeroUpdatePushCmd, push).await?;

        tracing::info!(
            "Sent HeroUpdatePush for hero {} with main equip {} and sub equip {}",
            hero_id,
            main_id,
            sub_id
        );

        conn.send_reply(CmdId::ChoiceHero3123WeaponCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
