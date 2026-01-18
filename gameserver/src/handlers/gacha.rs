use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::{
    BannerType, ConnectionContext, GachaResult, GachaState, build_gacha, grant_dupe_rewards,
    load_gacha_state, save_gacha_state,
};
use crate::util::push;
use data::exceldb;
use database::db::game::summon::{add_summon_history, get_sp_pool_info};
use std::sync::Arc;

use database::models::game::currencies::UserCurrencyModel;
use database::models::game::heros::UserHeroModel;
use database::models::game::items::UserItemModel;
use prost::Message;
use rand::thread_rng;
use tokio::sync::Mutex;

use crate::util::push::send_red_dot_push;

use database::db::{
    game::summon::{get_summon_pool_infos, get_summon_stats, *},
    user::account::get_user_token,
};

use sonettobuf::{
    ChooseEnhancedPoolHeroReply, ChooseEnhancedPoolHeroRequest, CmdId, EndActivityPush,
    GetSummonInfoReply, SummonQueryTokenReply, SummonReply, SummonRequest, SummonResult,
};

pub async fn on_get_summon_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (stats, pool_infos) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        let stats = get_summon_stats(&conn.state.db, player_id).await?;
        let pools = get_summon_pool_infos(&conn.state.db, player_id).await?;

        (stats, pools)
    };

    let reply = GetSummonInfoReply {
        free_equip_summon: Some(stats.free_equip_summon),
        is_show_new_summon: Some(stats.is_show_new_summon),
        new_summon_count: Some(stats.new_summon_count),
        pool_infos: pool_infos.into_iter().map(Into::into).collect(),
        total_summon_count: Some(stats.total_summon_count),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetSummonInfoCmd, reply, 0, req.up_tag)
        .await?;
    Ok(())
}

pub async fn on_choose_enhanced_pool_hero(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = ChooseEnhancedPoolHeroRequest::decode(&req.data[..])?;
    let pool_id = request.pool_id.ok_or(AppError::InvalidRequest)?;
    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        (player_id, conn.state.db.clone())
    };

    update_sp_pool_up_heroes(&pool, player_id, pool_id, &[hero_id]).await?;

    {
        let mut conn = ctx.lock().await;
        let reply = ChooseEnhancedPoolHeroReply {
            pool_id: Some(pool_id),
            hero_id: Some(hero_id),
        };

        conn.send_reply(CmdId::ChooseEnhancedPoolHeroCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}

pub async fn on_summon_query_token(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (player_id, pool) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        (player_id, conn.state.db.clone())
    };

    let token = get_user_token(&pool, player_id).await?;

    {
        let mut conn = ctx.lock().await;
        let push = EndActivityPush { id: Some(12716) };
        conn.notify(CmdId::EndActivityPushCmd, push).await?;
    }

    send_red_dot_push(Arc::clone(&ctx), player_id, Some(vec![1908])).await?;

    {
        let mut conn = ctx.lock().await;
        let reply = SummonQueryTokenReply {
            token: Some(token.token),
        };

        conn.send_reply(CmdId::SummonQueryTokenCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}

pub async fn on_summon(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SummonRequest::decode(&req.data[..])?;

    let pool_id = request.pool_id.unwrap_or(0);
    let count = request.count.unwrap_or(1).clamp(1, 10);

    tracing::info!("Summon request received: Pool {} Count {}", pool_id, count);

    let (user_id, db) = {
        let ctx = ctx.lock().await;
        (
            ctx.player_id.ok_or(AppError::NotLoggedIn)?,
            ctx.state.db.clone(),
        )
    };

    let hero = UserHeroModel::new(user_id, db.clone());
    let item = UserItemModel::new(user_id, db.clone());
    let currency = UserCurrencyModel::new(user_id, db.clone());

    let game_data = exceldb::get();
    let summon_pool = game_data
        .summon_pool
        .iter()
        .find(|p| p.id == pool_id)
        .ok_or(AppError::InvalidRequest)?;

    let is_discounted = if count == 10 {
        let pool_info: Option<(i32,)> = sqlx::query_as(
            "SELECT discount_time FROM user_summon_pools WHERE user_id = ? AND pool_id = ?",
        )
        .bind(user_id)
        .bind(pool_id)
        .fetch_optional(&db)
        .await?;

        pool_info
            .map(|(discount_time,)| discount_time > 0)
            .unwrap_or(false)
    } else {
        false
    };

    let cost_str = if is_discounted && !summon_pool.discount_cost10.is_empty() {
        &summon_pool.discount_cost10
    } else if count == 10 {
        &summon_pool.cost10
    } else {
        &summon_pool.cost1
    };

    let cost_options: Vec<&str> = cost_str.split('|').collect();

    let mut selected_items = Vec::new();
    let mut selected_currencies = Vec::new();

    for option in &cost_options {
        let (items, currencies, _, _, _, _) = crate::state::parse_store_product(option);

        let mut can_afford = true;
        for (item_id, amount) in &items {
            let current = item
                .get_item(*item_id)
                .await?
                .map(|i| i.quantity)
                .unwrap_or(0);
            if current < *amount {
                can_afford = false;
                break;
            }
        }

        if can_afford {
            selected_items = items;
            selected_currencies = currencies;
            break;
        }
    }

    if selected_items.is_empty() {
        let (items, currencies, _, _, _, _) =
            crate::state::parse_store_product(cost_options.last().unwrap());
        selected_items = items;
        selected_currencies = currencies;
    }

    let mut actual_cost_items = Vec::new();
    let mut actual_cost_currencies = selected_currencies.clone();
    let mut tickets_converted = 0;

    for (item_id, amount) in &selected_items {
        let current = item
            .get_item(*item_id)
            .await?
            .map(|i| i.quantity)
            .unwrap_or(0);

        if current >= *amount {
            actual_cost_items.push((*item_id, *amount));
        } else if *item_id == 140001 {
            let tickets_to_use = current;
            let shortage = *amount - current;
            let currency_needed = shortage * 180;

            let currency_balance = currency
                .get_currency(2)
                .await?
                .map(|c| c.quantity)
                .unwrap_or(0);

            if currency_balance < currency_needed {
                return Err(AppError::InsufficientCurrency);
            }

            if tickets_to_use > 0 {
                actual_cost_items.push((*item_id, tickets_to_use));
            }

            actual_cost_currencies.push((2, currency_needed));
            tickets_converted = shortage;
        } else {
            return Err(AppError::InsufficientItems);
        }
    }

    let needs_conversion = tickets_converted > 0;

    if needs_conversion {
        push::send_item_change_push(ctx.clone(), user_id, vec![140001], vec![], vec![]).await?;
    }

    {
        for (item_id, amount) in &actual_cost_items {
            item.remove_item_quantity(*item_id, *amount).await?;
        }

        for (currency_id, amount) in &actual_cost_currencies {
            currency.remove_currency(*currency_id, *amount).await?;
        }
    }

    let pool_cfg = exceldb::get()
        .summon_pool
        .iter()
        .find(|p| p.id == pool_id)
        .expect("Summon pool not found");

    let banner_type = BannerType::from(pool_cfg.r#type);

    let sp_pool_info = get_sp_pool_info(&db, user_id, pool_id).await?;

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
    let mut all_changed_item_ids: Vec<u32> = Vec::new();
    let mut all_changed_currencies = Vec::new();
    let mut new_heroes = Vec::new();

    for result in gacha_results {
        match result {
            GachaResult::Hero {
                hero_id,
                rare,
                is_up,
            } => {
                let (is_new, duplicate_count) = if hero.has_hero(hero_id).await? {
                    let dup = hero.add_hero_duplicate(hero_id).await?;
                    (false, dup)
                } else {
                    hero.create_hero(hero_id).await?;
                    new_heroes.push(hero_id);
                    (true, 0)
                };

                if !is_new && duplicate_count > 0 {
                    let (item_rewards, currency_rewards) =
                        grant_dupe_rewards(hero_id, duplicate_count).await?;

                    if !item_rewards.is_empty() {
                        let item_ids = item.create_items(&item_rewards).await?;
                        all_changed_item_ids.extend(item_ids.iter().map(|id| *id as u32));
                    }

                    if !currency_rewards.is_empty() {
                        let currency_changes =
                            currency.create_currencies(&currency_rewards).await?;
                        all_changed_currencies.extend(currency_changes);
                    }
                }

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

    if is_discounted {
        database::db::game::summon::use_discount(&db, user_id, pool_id).await?;
    }

    if let Err(e) =
        database::db::game::summon::increment_summon_count(&db, user_id, pool_id, count).await
    {
        tracing::warn!("Failed to increment summon count: {}", e);
    }

    all_changed_item_ids.extend(actual_cost_items.iter().map(|(id, _)| *id));

    if !all_changed_item_ids.is_empty() {
        push::send_item_change_push(ctx.clone(), user_id, all_changed_item_ids, vec![], vec![])
            .await?;
    }

    if !all_changed_currencies.is_empty() || !actual_cost_currencies.is_empty() {
        all_changed_currencies.extend(actual_cost_currencies.iter().map(|(id, _)| (*id, 0)));
        push::send_currency_change_push(ctx.clone(), user_id, all_changed_currencies).await?;
    }

    if !new_heroes.is_empty() {
        let mut hero_infos = Vec::new();
        for hero_id in new_heroes {
            if let Ok(hero) = hero.get_hero(hero_id).await {
                hero_infos.push(hero.into());
            }
        }

        if !hero_infos.is_empty() {
            let hero_push = sonettobuf::HeroUpdatePush {
                hero_updates: hero_infos,
            };

            let mut conn = ctx.lock().await;
            conn.notify(CmdId::HeroHeroUpdatePushCmd, hero_push).await?;
            drop(conn);
        }
    }

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
