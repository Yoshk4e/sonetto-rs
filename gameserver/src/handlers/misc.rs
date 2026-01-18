use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::{
    db::game::bgm::{load_user_bgm, set_active_bgm, set_bgm_favorite},
    models::game::heros::{HeroModel, UserHeroModel},
};
use prost::Message;
use sonettobuf::{
    CmdId, GetBgmInfoReply, HeroUpdatePush, SetFavoriteBgmReply, SetFavoriteBgmRequest,
    SetPortraitRequest, SetUseBgmReply, SetUseBgmRequest, UseSkinReply, UseSkinRequest,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_bgm_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (bgm_infos, use_bgm_id) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        load_user_bgm(&conn.state.db, player_id).await?
    };

    let reply = GetBgmInfoReply {
        bgm_infos,
        use_bgm_id,
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetBgmInfoCmd, reply, 0, req.up_tag)
        .await?;
    Ok(())
}

pub async fn on_set_favorite_bgm(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SetFavoriteBgmRequest::decode(&req.data[..])?;

    let mut conn = ctx.lock().await;
    let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
    let pool = &conn.state.db;

    set_bgm_favorite(
        pool,
        player_id,
        request.bgm_id.unwrap_or(2207),
        request.favorite.unwrap_or(false),
    )
    .await
    .map_err(AppError::from)?;

    let reply = SetFavoriteBgmReply {
        bgm_id: request.bgm_id,
        favorite: request.favorite,
    };

    conn.send_reply(CmdId::SetFavoriteBgmCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_set_use_bgm(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SetUseBgmRequest::decode(&req.data[..])?;

    let mut conn = ctx.lock().await;
    let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
    let pool = &conn.state.db;

    set_active_bgm(pool, player_id, request.bgm_id.unwrap_or(2207))
        .await
        .map_err(AppError::from)?;

    let reply = SetUseBgmReply {
        bgm_id: request.bgm_id,
    };

    conn.send_reply(CmdId::SetUseBgmCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_use_skin(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UseSkinRequest::decode(&req.data[..])?;
    tracing::info!("Received UseSkinRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let skin_id = request.skin_id.ok_or(AppError::InvalidRequest)?;

    let updated_hero = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let hero = UserHeroModel::new(player_id, pool.clone());
        let hero_data = hero.get(hero_id).await?;
        let hero_info: sonettobuf::HeroInfo = hero_data.into();

        hero.update_skin(hero_id, skin_id).await?;

        tracing::info!(
            "User {} equipped skin {} on hero {}",
            player_id,
            skin_id,
            hero_id
        );
        hero_info
    };

    let data = UseSkinReply {
        hero_id: Some(hero_id),
        skin_id: Some(skin_id),
    };

    {
        let mut conn = ctx.lock().await;

        let hero_proto: sonettobuf::HeroInfo = updated_hero;
        let push = HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };

        conn.notify(CmdId::HeroHeroUpdatePushCmd, push).await?;

        conn.send_reply(CmdId::UseSkinCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}

pub async fn on_set_portrait(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SetPortraitRequest::decode(&req.data[..])?;
    tracing::info!("Received SetPortraitRequest: {:?}", request);

    let portrait = request.portrait.ok_or(AppError::InvalidRequest)?;

    let _ = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        sqlx::query("UPDATE player_info SET portrait = ? WHERE player_id = ?")
            .bind(portrait)
            .bind(player_id)
            .execute(pool)
            .await?;

        tracing::info!("User {} updated portrait to {}", player_id, portrait);

        player_id
    };

    let mut conn = ctx.lock().await;
    conn.send_empty_reply(CmdId::SetPortraitCmd, Vec::new(), 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_mark_main_thumbnail(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let mut conn = ctx.lock().await;
    conn.send_empty_reply(CmdId::MarkMainThumbnailCmd, Vec::new(), 0, req.up_tag)
        .await?;

    Ok(())
}
