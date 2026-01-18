use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;

use database::models::game::{
    heros::{HeroModel, UserHeroModel},
    items::UserItemModel,
};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, HeroUpgradeSkillReply, HeroUpgradeSkillRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_upgrade_skill(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroUpgradeSkillRequest::decode(&req.data[..])?;
    let hero_id = request.hero_id;
    let skill_type = request.r#type; // 3 = ex_skill
    let consume = request.consume.unwrap_or(1);

    tracing::info!("Received HeroUpgradeSkillRequest: {:?}", request);

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = conn.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(player_id, pool.clone());
    let item = UserItemModel::new(player_id, pool);

    let consumed_item_id = {
        let hero_data = hero.get(hero_id).await?;

        // Check current skill level
        if skill_type == 3 && hero_data.record.ex_skill_level >= 5 {
            return Err(AppError::InvalidRequest);
        }

        let game_data = data::exceldb::get();
        let character = game_data
            .character
            .iter()
            .find(|c| c.id == hero_id)
            .ok_or(AppError::InvalidRequest)?;

        // Parse duplicateItem: "1#133125#1|2#11#12"
        let dupe_item_id = character
            .duplicate_item
            .split('|')
            .next()
            .and_then(|part| {
                let segments: Vec<&str> = part.split('#').collect();
                if segments.len() >= 3 && segments[0] == "1" {
                    segments[1].parse::<i32>().ok()
                } else {
                    None
                }
            })
            .ok_or(AppError::InvalidRequest)?;

        let success = item
            .remove_item_quantity(dupe_item_id as u32, consume)
            .await?;
        if !success {
            return Err(AppError::InsufficientItems);
        }

        if skill_type == 3 {
            hero.upgrade_ex_skill(hero_id, consume).await?;

            tracing::info!(
                "User {} upgraded ex_skill by {} levels on hero {}",
                player_id,
                consume,
                hero_id
            );
        }

        dupe_item_id
    };

    crate::util::push::send_item_change_push(
        ctx.clone(),
        player_id,
        vec![consumed_item_id as u32],
        vec![],
        vec![],
    )
    .await?;

    {
        let conn = ctx.lock().await;
        let pool = &conn.state.db;
        let hero = UserHeroModel::new(player_id, pool.clone());
        let updated_hero = hero.get(hero_id).await?;

        drop(conn);

        let hero_proto: sonettobuf::HeroInfo = updated_hero.into();
        let push = HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };

        let mut conn = ctx.lock().await;
        conn.notify(CmdId::HeroHeroUpdatePushCmd, push).await?;

        tracing::info!("Sent HeroUpdatePush for hero {} ex_skill upgrade", hero_id);
    }

    let data = HeroUpgradeSkillReply {};
    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::HeroUpgradeSkillCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
