use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, TalentStyleReadReply, TalentStyleReadRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_talent_style_stat(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = TalentStyleReadRequest::decode(&req.data[..])?;
    tracing::info!("Received TalentStyleReadRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let data = TalentStyleReadReply {
        hero_id: Some(hero_id),
    };

    let hero = UserHeroModel::new(player_id, pool.clone());
    let hero_data = hero.get(hero_id).await?;

    hero.talent_style_read(hero_id).await?;

    tracing::info!(
        "User {} marked talent style as read for hero {}",
        player_id,
        hero_id
    );

    {
        let mut ctx_guard = ctx.lock().await;

        let hero_info: sonettobuf::HeroInfo = hero_data.into();
        ctx_guard
            .send_push(
                CmdId::HeroHeroUpdatePushCmd,
                HeroUpdatePush {
                    hero_updates: vec![hero_info.into()],
                },
            )
            .await?;

        ctx_guard
            .send_reply(CmdId::HeroTalentStyleStatCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
