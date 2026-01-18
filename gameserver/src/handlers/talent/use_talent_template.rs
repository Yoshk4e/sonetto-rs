use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, UseTalentTemplateReply, UseTalentTemplateRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_use_talent_template(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UseTalentTemplateRequest::decode(&req.data[..])?;
    tracing::info!("Received UseTalentTemplateRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let template_id = request.template_id.ok_or(AppError::InvalidRequest)?;

    let (user_id, pool) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = conn.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(user_id, pool);
    let template_info = hero.switch_talent_template(hero_id, template_id).await?;

    let data = UseTalentTemplateReply {
        hero_id: Some(hero_id),
        template_info: Some(template_info),
    };

    {
        let updated_hero = hero.get(hero_id).await?;
        let hero_info: sonettobuf::HeroInfo = updated_hero.into();

        let mut conn = ctx.lock().await;
        conn.notify(
            CmdId::HeroHeroUpdatePushCmd,
            HeroUpdatePush {
                hero_updates: vec![hero_info],
            },
        )
        .await?;

        conn.send_reply(CmdId::UseTalentTemplateCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
