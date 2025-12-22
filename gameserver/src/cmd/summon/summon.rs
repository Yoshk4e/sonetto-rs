use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::{
    BannerType, ConnectionContext, GachaResult, GachaState, build_gacha, load_gacha_state,
    save_gacha_state,
};
use data::exceldb;
use database::db::game::heroes::{add_hero_duplicate, create_hero, has_hero};
use database::db::game::summon::{add_summon_history, get_sp_pool_info};
use prost::Message;
use rand::thread_rng;

use sonettobuf::{CmdId, SummonReply, SummonRequest, SummonResult};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_summon(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SummonRequest::decode(&req.data[..])?;

    let pool_id = request.pool_id.unwrap_or(0);
    let count = request.count.unwrap_or(1).clamp(1, 10);

    let (user_id, db) = {
        let ctx = ctx.lock().await;
        (
            ctx.player_id.ok_or(AppError::NotLoggedIn)?,
            ctx.state.db.clone(),
        )
    };

    let sp_pool_info = get_sp_pool_info(&db, user_id, pool_id).await?;

    let banner_type = match &sp_pool_info {
        Some(sp) => BannerType::from(sp.sp_type),
        None => BannerType::Normal,
    };

    let pool = build_gacha(pool_id, sp_pool_info.as_ref()).await?;

    let state = load_gacha_state(&db, user_id, pool_id).await?;
    let mut gacha = GachaState {
        pity_6: state.pity_6,
        up_guaranteed: state.up_guaranteed,
    };

    let gacha_results = {
        let mut rng = thread_rng();
        if count == 10 {
            gacha.ten_pull(banner_type, &pool, &mut rng)
        } else {
            vec![gacha.single_pull(banner_type, &pool, &mut rng, false)]
        }
    };

    let mut reply_results = Vec::with_capacity(gacha_results.len());

    for result in gacha_results {
        match result {
            GachaResult::Hero {
                hero_id,
                rare,
                is_up,
            } => {
                let (is_new, duplicate_count) = if has_hero(&db, user_id, hero_id).await? {
                    let dup = add_hero_duplicate(&db, user_id, hero_id).await?;
                    (false, dup)
                } else {
                    create_hero(&db, user_id, hero_id).await?;
                    (true, 0)
                };

                reply_results.push(SummonResult {
                    hero_id: Some(hero_id),
                    is_new: Some(is_new),
                    duplicate_count: Some(duplicate_count),
                    equip_id: Some(0),
                    return_materials: Vec::new(),
                    lucky_bag_id: Some(0),
                    limited_ticket_id: Some(0),
                });

                tracing::info!(
                    "User {} pulled hero {} (rarity: {}, is_up: {}, is_new: {})",
                    user_id,
                    hero_id,
                    rare,
                    is_up,
                    is_new
                );
            }
        }
    }

    save_gacha_state(&db, user_id, pool_id, &gacha).await?;

    let game_data = exceldb::get();
    let summon_pool = game_data
        .summon_pool
        .iter()
        .find(|p| p.id == pool_id)
        .ok_or(AppError::InvalidRequest)?;

    let summon_type = if count == 10 { 2 } else { 1 };

    add_summon_history(
        &db,
        user_id,
        pool_id,
        summon_pool.name_en.as_str(),
        summon_pool.r#type,
        summon_type,
        &reply_results,
    )
    .await?;

    let reply = SummonReply {
        summon_result: reply_results,
    };

    {
        let mut ctx = ctx.lock().await;
        ctx.send_reply(CmdId::SummonCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
