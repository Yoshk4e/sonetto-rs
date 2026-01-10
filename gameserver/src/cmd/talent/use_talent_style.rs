use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, UseTalentStyleReply, UseTalentStyleRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_use_talent_style(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UseTalentStyleRequest::decode(&req.data[..])?;
    tracing::info!("Received UseTalentStyleRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let template_id = request.template_id.ok_or(AppError::InvalidRequest)?;
    let style = request.style.ok_or(AppError::InvalidRequest)?;

    let (user_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(user_id, pool);
    hero.apply_talent_style(hero_id, template_id, style).await?;

    let data = UseTalentStyleReply {
        hero_id: Some(hero_id),
        template_id: Some(template_id),
        style: Some(style),
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
            .send_reply(CmdId::UseTalentStyleCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
