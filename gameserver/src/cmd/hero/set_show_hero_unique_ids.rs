use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::{
    db::game::player_infos::get_player_info_data,
    models::game::heros::{HeroModel, UserHeroModel},
};
use prost::Message;
use sonettobuf::{CmdId, PlayerInfoPush, SetShowHeroUniqueIdsReply, SetShowHeroUniqueIdsRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_set_show_hero_unique_ids(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SetShowHeroUniqueIdsRequest::decode(&req.data[..])?;
    let hero_uids = request.show_hero_unique_ids;

    let mut ctx_guard = ctx.lock().await;
    let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
    let pool = &ctx_guard.state.db;

    let hero = UserHeroModel::new(player_id, pool.clone());

    hero.set_show_hero(&hero_uids)
        .await
        .map_err(AppError::from)?;

    let player_info_data = get_player_info_data(pool, player_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotLoggedIn)?;

    let player_info = player_info_data.into();

    tracing::info!("Sending PlayerInfoPush update");
    ctx_guard
        .send_push(
            CmdId::PlayerInfoPushCmd,
            PlayerInfoPush {
                player_info: Some(player_info),
            },
        )
        .await?;

    ctx_guard
        .send_reply(
            CmdId::SetShowHeroUniqueIdsCmd,
            SetShowHeroUniqueIdsReply {},
            0,
            req.up_tag,
        )
        .await?;

    Ok(())
}
