use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, PutTalentSchemeReply, PutTalentSchemeRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_put_talent_scheme(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = PutTalentSchemeRequest::decode(&req.data[..])?;
    tracing::info!("Received PutTalentSchemeRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let talent_id = request.talent_id.ok_or(AppError::InvalidRequest)?;
    let talent_mould = request.talent_mould.ok_or(AppError::InvalidRequest)?;
    let template_id = request.template_id.ok_or(AppError::InvalidRequest)?;

    let (user_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(user_id, pool);

    let template_info = hero
        .load_talent_scheme(hero_id, talent_id, talent_mould, template_id)
        .await?;

    let data = PutTalentSchemeReply {
        hero_id: Some(hero_id),
        template_info: Some(template_info),
    };

    {
        let updated_hero = hero.get(hero_id).await?;
        let hero_info: sonettobuf::HeroInfo = updated_hero.into();

        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(
                CmdId::HeroHeroUpdatePushCmd,
                HeroUpdatePush {
                    hero_updates: vec![hero_info],
                },
            )
            .await?;

        ctx_guard
            .send_reply(CmdId::PutTalentSchemeCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
