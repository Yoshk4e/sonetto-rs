use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::heros::{HeroModel, UserHeroModel};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, PutTalentCubeReply, PutTalentCubeRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_put_talent_cube(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = PutTalentCubeRequest::decode(&req.data[..])?;
    tracing::info!("Received PutTalentCubeRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let get_cube_info = request.get_cube_info;
    let put_cube_info = request.put_cube_info;
    let template_id = request.template_id.ok_or(AppError::InvalidRequest)?;

    let (user_id, pool) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = conn.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(user_id, pool);

    if let Some(get_cube) = &get_cube_info {
        let pos_x = get_cube.pos_x.unwrap_or(0);
        let pos_y = get_cube.pos_y.unwrap_or(0);

        hero.remove_talent_cube(hero_id, template_id, pos_x, pos_y)
            .await?;

        tracing::info!(
            "Removed cube from template {} at ({}, {})",
            template_id,
            pos_x,
            pos_y
        );
    }

    if let Some(put_cube) = &put_cube_info {
        let cube_id = put_cube.cube_id.ok_or(AppError::InvalidRequest)?;
        let direction = put_cube.direction.ok_or(AppError::InvalidRequest)?;
        let pos_x = put_cube.pos_x.ok_or(AppError::InvalidRequest)?;
        let pos_y = put_cube.pos_y.ok_or(AppError::InvalidRequest)?;

        hero.place_talent_cube(hero_id, template_id, cube_id, direction, pos_x, pos_y)
            .await?;

        tracing::info!(
            "Placed cube {} at ({}, {}) in template {}",
            cube_id,
            pos_x,
            pos_y,
            template_id
        );
    }

    let get_cube_pos = get_cube_info
        .as_ref()
        .map(|c| (c.pos_x.unwrap_or(0), c.pos_y.unwrap_or(0)));
    let put_cube_data = put_cube_info.as_ref().map(|c| {
        (
            c.cube_id.unwrap(),
            c.direction.unwrap(),
            c.pos_x.unwrap(),
            c.pos_y.unwrap(),
        )
    });

    hero.sync_active_talent_cubes(hero_id, template_id, get_cube_pos, put_cube_data)
        .await?;

    let template_info = hero.get_template_info(hero_id, template_id).await?;

    let data = PutTalentCubeReply {
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

        conn.send_reply(CmdId::PutTalentCubeCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
