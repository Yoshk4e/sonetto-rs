use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, TalentStyleReadReply, TalentStyleReadRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_talent_style_read(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = TalentStyleReadRequest::decode(&req.data[..])?;
    tracing::info!("Received TalentStyleReadRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (user_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(user_id, pool);

    let data = TalentStyleReadReply {
        hero_id: Some(hero_id),
    };

    {
        let mut ctx_guard = ctx.lock().await;

        let updated_hero = hero.get(hero_id).await?;
        let hero_info: sonettobuf::HeroInfo = updated_hero.into();

        ctx_guard
            .send_push(
                CmdId::HeroHeroUpdatePushCmd,
                HeroUpdatePush {
                    hero_updates: vec![hero_info],
                },
            )
            .await?;

        ctx_guard
            .send_reply(CmdId::TalentStyleReadCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
