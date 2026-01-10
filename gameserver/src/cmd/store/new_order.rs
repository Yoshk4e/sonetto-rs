use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;

use crate::utils::push;
use database::models::game::{
    currencies::UserCurrencyModel, heros::UserHeroModel, items::UserItemModel,
};

use prost::Message;
use sonettobuf::{CmdId, NewOrderReply, NewOrderRequest, OrderCompletePush, StatInfoPush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_new_order(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = NewOrderRequest::decode(&req.data[..])?;
    tracing::info!("Received NewOrderRequest: {:?}", request);

    let goods_id = request.id.ok_or(AppError::InvalidRequest)?;
    let selection_infos = request.selection_infos;

    let now = common::time::ServerTime::now_ms();
    let game_order_id = now;

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(player_id, pool.clone());
    let item = UserItemModel::new(player_id, pool.clone());
    let currency = UserCurrencyModel::new(player_id, pool.clone());

    let (
        user_id,
        attachment,
        changed_items,
        changed_currencies,
        first_charge,
        total_charge,
        user_tag,
        _,
    ) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let game_data = data::exceldb::get();
        let charge_pack = game_data
            .store_charge_goods
            .iter()
            .find(|g| g.id == goods_id)
            .ok_or(AppError::InvalidRequest)?;

        let is_month_card = charge_pack.r#type == 2;

        let all_items = if is_month_card {
            if let Some(month_card) = game_data.month_card.iter().find(|m| m.id == goods_id) {
                let existing_end_time: Option<i64> = sqlx::query_scalar(
                    "SELECT end_time FROM user_month_card_history
                     WHERE user_id = ? AND card_id = ?
                     ORDER BY end_time DESC LIMIT 1",
                )
                .bind(player_id)
                .bind(goods_id)
                .fetch_optional(pool)
                .await?;

                let start_time = now;
                let days_to_add = month_card.days as i64;

                let new_end_time = if let Some(existing_end) = existing_end_time {
                    let existing_end_ms = existing_end * 1000;
                    let base_time = if existing_end_ms > now {
                        existing_end_ms
                    } else {
                        now
                    };
                    base_time + (days_to_add * 24 * 60 * 60 * 1000)
                } else {
                    now + (days_to_add * 24 * 60 * 60 * 1000)
                };

                if existing_end_time.is_some() {
                    sqlx::query(
                        "UPDATE user_month_card_history
                         SET end_time = ?
                         WHERE user_id = ? AND card_id = ?
                         AND end_time = (SELECT MAX(end_time) FROM user_month_card_history WHERE user_id = ? AND card_id = ?)"
                    )
                    .bind(new_end_time / 1000)
                    .bind(player_id)
                    .bind(goods_id)
                    .bind(player_id)
                    .bind(goods_id)
                    .execute(pool)
                    .await?;
                } else {
                    sqlx::query(
                        "INSERT INTO user_month_card_history (user_id, card_id, start_time, end_time)
                         VALUES (?, ?, ?, ?)",
                    )
                    .bind(player_id)
                    .bind(goods_id)
                    .bind(start_time / 1000)
                    .bind(new_end_time / 1000)
                    .execute(pool)
                    .await?;
                }

                let current_server_day = common::time::ServerTime::server_day(now);
                sqlx::query(
                    "INSERT OR IGNORE INTO user_month_card_days (user_id, server_day, day_of_month)
                     VALUES (?, ?, 1)",
                )
                .bind(player_id)
                .bind(current_server_day)
                .execute(pool)
                .await?;

                tracing::info!(
                    "User {} activated month card {} for {} days (expires: {}, extended: {})",
                    player_id,
                    goods_id,
                    days_to_add,
                    new_end_time,
                    existing_end_time.is_some()
                );

                format!("{}|{}", month_card.once_bonus, month_card.daily_bonus)
            } else {
                String::new()
            }
        } else {
            let mut all_items = charge_pack.item.clone();

            for selection in &selection_infos {
                let region_id = selection.region_id.unwrap_or(0);
                let selection_pos = selection.selection_pos.unwrap_or(0);

                if let Some(optional) = game_data
                    .store_charge_optional
                    .iter()
                    .find(|o| o.goods_id == goods_id && o.id == region_id)
                {
                    let item_choices: Vec<&str> = optional.items.split('|').collect();

                    if let Some(selected_item) = item_choices.get(selection_pos as usize) {
                        if !all_items.is_empty() {
                            all_items.push('|');
                        }
                        all_items.push_str(selected_item);

                        tracing::info!(
                            "User {} selected option region={} pos={}: {}",
                            player_id,
                            region_id,
                            selection_pos,
                            selected_item
                        );
                    }
                }
            }

            all_items
        };

        let attachment = all_items.clone();

        let (items, currencies, equips, heroes, power_items, insight_selectors) =
            crate::state::parse_store_product(&all_items);

        let mut item_ids = if !items.is_empty() {
            item.create_items(&items).await?
        } else {
            vec![]
        };

        let currency_ids = if !currencies.is_empty() {
            currency.create_currencies(&currencies).await?
        } else {
            vec![]
        };

        if !power_items.is_empty() {
            item.create_power_items(
                &power_items
                    .iter()
                    .map(|(id, count)| (*id as i32, *count))
                    .collect::<Vec<_>>(),
            )
            .await?;
        }

        if !insight_selectors.is_empty() {
            let insight_item_ids = item
                .create_insight_items(
                    &insight_selectors
                        .iter()
                        .map(|(id, count)| (*id as i32, *count))
                        .collect::<Vec<_>>(),
                )
                .await?;
            item_ids.extend(insight_item_ids);
        }

        if !equips.is_empty() {
            database::db::game::equipment::add_equipments(
                pool,
                player_id,
                &equips
                    .iter()
                    .map(|(id, count)| (*id as i32, *count))
                    .collect::<Vec<_>>(),
            )
            .await?;
        }

        for (hero_id, _count) in &heroes {
            let hero_id = *hero_id as i32;
            if !hero.has_hero(hero_id).await? {
                hero.create_hero(hero_id).await?;
                tracing::info!(
                    "User {} received new hero {} from charge pack",
                    player_id,
                    hero_id
                );
            }
        }

        let (current_buy_count, current_first_charge): (i32, i32) = sqlx::query_as(
            "SELECT buy_count, first_charge FROM user_charge_info WHERE user_id = ? AND charge_id = ?"
        )
        .bind(player_id)
        .bind(goods_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or((0, 0));

        let new_buy_count = current_buy_count + 1;
        let new_first_charge = if current_first_charge == 0 && new_buy_count == 1 {
            1
        } else {
            current_first_charge
        };

        sqlx::query(
            "UPDATE user_charge_info
             SET buy_count = ?, first_charge = ?, updated_at = ?
             WHERE user_id = ? AND charge_id = ?",
        )
        .bind(new_buy_count)
        .bind(new_first_charge)
        .bind(now)
        .bind(player_id)
        .bind(goods_id)
        .execute(pool)
        .await?;

        let (stats_first_charge, mut total_charge_amount, user_tag): (i32, i64, String) = sqlx::query_as(
            "SELECT first_charge, total_charge_amount, user_tag FROM user_stats WHERE user_id = ?"
        )
        .bind(player_id)
        .fetch_one(pool)
        .await?;

        let charge_amount = (charge_pack.price * 100.0) as i64;
        total_charge_amount += charge_amount;

        let new_stats_first_charge = if stats_first_charge == 0 {
            1
        } else {
            stats_first_charge
        };

        sqlx::query(
            "UPDATE user_stats
             SET first_charge = ?, total_charge_amount = ?
             WHERE user_id = ?",
        )
        .bind(new_stats_first_charge)
        .bind(total_charge_amount)
        .bind(player_id)
        .execute(pool)
        .await?;

        tracing::info!(
            "User {} purchased charge pack {} (price: ${}, total: ${:.2}, first_charge: {}, month_card: {})",
            player_id,
            goods_id,
            charge_pack.price,
            total_charge_amount as f64 / 100.0,
            new_stats_first_charge,
            is_month_card
        );

        (
            player_id,
            attachment,
            item_ids,
            currency_ids,
            new_stats_first_charge == 1,
            total_charge_amount,
            user_tag,
            is_month_card,
        )
    };

    let reply = NewOrderReply {
        id: Some(goods_id),
        pass_back_param: Some("".to_string()),
        notify_url: Some("".to_string()),
        game_order_id: Some(game_order_id),
        timestamp: Some(now),
        sign: Some("03f92726ce15e0793dddd7f1a9db39f28".to_string()),
        server_id: Some(4),
        currency: request.origin_currency.clone(),
    };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::NewOrderCmd, reply, 0, req.up_tag)
            .await?;
    }

    if !changed_items.is_empty() {
        push::send_item_change_push(
            ctx.clone(),
            user_id,
            changed_items.into_iter().map(|id| id as u32).collect(),
            vec![],
            vec![],
        )
        .await?;
    }

    if !changed_currencies.is_empty() {
        push::send_currency_change_push(ctx.clone(), user_id, changed_currencies).await?;
    }

    let complete_push = OrderCompletePush {
        id: Some(goods_id),
        game_order_id: Some(game_order_id),
    };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(CmdId::OrderCompletePushCmd, complete_push)
            .await?;
    }

    let stat_push = StatInfoPush {
        frist_charge: Some(first_charge),
        total_charge_amount: Some(total_charge),
        is_first_login: Some(false),
        player_info: None,
        user_tag: Some(user_tag),
    };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(CmdId::StatInfoPushCmd, stat_push)
            .await?;
    }

    let mut material_changes = Vec::new();
    let (items, currencies, equips, heroes, power_items, insight_selectors) =
        crate::state::parse_store_product(&attachment);

    for (item_id, amount) in items {
        material_changes.push((1, item_id, amount));
    }
    for (currency_id, amount) in currencies {
        material_changes.push((2, currency_id as u32, amount));
    }
    for (equip_id, amount) in equips {
        material_changes.push((9, equip_id, amount));
    }
    for (hero_id, amount) in heroes {
        material_changes.push((4, hero_id, amount));
    }
    for (power_item_id, amount) in power_items {
        material_changes.push((10, power_item_id, amount));
    }
    for (insight_selector_id, amount) in insight_selectors {
        material_changes.push((24, insight_selector_id, amount));
    }

    if !material_changes.is_empty() {
        push::send_material_change_push(ctx.clone(), material_changes, Some(27)).await?;
    }

    tracing::info!(
        "Auto-completed order {} for goods {}",
        game_order_id,
        goods_id
    );

    Ok(())
}
