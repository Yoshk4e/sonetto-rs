use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, UnMarkIsNewReply, UnMarkIsNewRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_unmark_is_new(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UnMarkIsNewRequest::decode(&req.data[..])?;
    tracing::info!("Received UnMarkIsNewRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = conn.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(player_id, pool);
    hero.unmark_new(hero_id).await?;

    tracing::info!("User {} unmarked hero {} as new", player_id, hero_id);

    let data = UnMarkIsNewReply {
        hero_id: Some(hero_id),
    };

    {
        let updated_hero = hero.get(hero_id).await?;

        let hero_proto: sonettobuf::HeroInfo = updated_hero.into();
        let push = HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };

        let mut conn = ctx.lock().await;
        conn.notify(CmdId::HeroHeroUpdatePushCmd, push).await?;

        conn.send_reply(CmdId::UnMarkIsNewCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
