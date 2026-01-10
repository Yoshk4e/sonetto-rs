use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;

use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroTouchReply, HeroTouchRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_touch(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroTouchRequest::decode(&req.data[..])?;
    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (success, touch_count_left) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let hero = UserHeroModel::new(player_id, pool.clone());

        match hero.use_touch().await? {
            Some(new_count) => {
                tracing::info!(
                    "User {} touched hero {}, {} touches remaining",
                    player_id,
                    hero_id,
                    new_count
                );
                (true, new_count)
            }
            None => {
                tracing::warn!(
                    "User {} tried to touch hero {} but has no touches left",
                    player_id,
                    hero_id
                );
                (false, 0)
            }
        }
    };

    let data = HeroTouchReply {
        touch_count_left: Some(touch_count_left),
        success: Some(success),
    };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::HeroTouchCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
