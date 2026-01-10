use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::{ConnectionContext, parse_store_product};
use crate::utils::push;
use data::exceldb;
use database::models::game::currencies::UserCurrencyModel;
use database::models::game::heros::UserHeroModel;
use database::models::game::items::UserItemModel;
use prost::Message;
use sonettobuf::{BuyGoodsReply, BuyGoodsRequest, CmdId};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_buy_goods(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = BuyGoodsRequest::decode(&req.data[..])?;
    let store_id = request.store_id;
    let goods_id = request.goods_id;
    let num = request.num;
    let select_cost = request.select_cost;

    tracing::info!(
        "Requested store_id: {:?}, goods_id: {:?}, num: {:?}, select_cost: {:?}",
        store_id,
        goods_id,
        num,
        select_cost
    );

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(player_id, pool.clone());
    let item = UserItemModel::new(player_id, pool.clone());
    let currency = UserCurrencyModel::new(player_id, pool.clone());

    let game_data = exceldb::get();
    let goods = game_data
        .store_goods
        .iter()
        .find(|g| g.id == goods_id)
        .ok_or_else(|| {
            tracing::error!("Goods {} not found", goods_id);
            AppError::InvalidRequest
        })?;

    let quantity = num;

    {
        let ctx_guard = ctx.lock().await;
        let pool = &ctx_guard.state.db;

        let current_buy_count: i32 = sqlx::query_scalar(
            "SELECT buy_count FROM user_store_goods WHERE user_id = ? AND goods_id = ?",
        )
        .bind(player_id)
        .bind(goods_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or(0);

        if goods.max_buy_count > 0 && current_buy_count + quantity > goods.max_buy_count {
            tracing::info!(
                "User {} at max buy count for goods {} (has {}, trying to buy {}, max {})",
                player_id,
                goods_id,
                current_buy_count,
                quantity,
                goods.max_buy_count
            );

            drop(ctx_guard);

            let reply = BuyGoodsReply {
                store_id,
                goods_id,
                num,
                select_cost,
            };

            let mut ctx_guard = ctx.lock().await;
            ctx_guard
                .send_reply(CmdId::BuyGoodsCmd, reply, 0, req.up_tag)
                .await?;

            return Ok(());
        }
    }

    let cost_str = if !goods.cost2.is_empty() && select_cost == Some(2) {
        &goods.cost2
    } else {
        &goods.cost
    };

    let (cost_items, cost_currencies, _, _, _, _) = parse_store_product(cost_str);
    let cost_items: Vec<(u32, i32)> = cost_items
        .iter()
        .map(|(id, amt)| (*id, amt * quantity))
        .collect();
    let cost_currencies: Vec<(i32, i32)> = cost_currencies
        .iter()
        .map(|(id, amt)| (*id, amt * quantity))
        .collect();

    {
        let ctx_guard = ctx.lock().await;
        let pool = &ctx_guard.state.db;

        for (item_id, amount) in &cost_items {
            let current = item
                .get_item(*item_id)
                .await?
                .map(|i| i.quantity)
                .unwrap_or(0);

            if current < *amount {
                tracing::info!(
                    "User {} insufficient item {} (has {}, needs {})",
                    player_id,
                    item_id,
                    current,
                    amount
                );

                drop(ctx_guard);

                push::send_item_change_push(ctx.clone(), player_id, vec![*item_id], vec![], vec![])
                    .await?;

                let mut ctx_guard = ctx.lock().await;
                ctx_guard
                    .send_reply(
                        CmdId::BuyGoodsCmd,
                        BuyGoodsReply {
                            store_id,
                            goods_id,
                            num,
                            select_cost,
                        },
                        0,
                        req.up_tag,
                    )
                    .await?;

                return Ok(());
            }
        }

        for (currency_id, amount) in &cost_currencies {
            let current = currency
                .get_currency(*currency_id)
                .await?
                .map(|c| c.quantity)
                .unwrap_or(0);

            if current < *amount {
                tracing::info!(
                    "User {} insufficient currency {} (has {}, needs {})",
                    player_id,
                    currency_id,
                    current,
                    amount
                );

                drop(ctx_guard);

                push::send_currency_change_push(ctx.clone(), player_id, vec![(*currency_id, 0)])
                    .await?;

                let mut ctx_guard = ctx.lock().await;
                ctx_guard
                    .send_reply(
                        CmdId::BuyGoodsCmd,
                        BuyGoodsReply {
                            store_id,
                            goods_id,
                            num,
                            select_cost,
                        },
                        0,
                        req.up_tag,
                    )
                    .await?;

                return Ok(());
            }
        }

        for (item_id, amount) in &cost_items {
            item.remove_item_quantity(*item_id, *amount).await?;
        }

        for (currency_id, amount) in &cost_currencies {
            currency.remove_currency(*currency_id, *amount).await?;
        }

        sqlx::query(
            "INSERT INTO user_store_goods (user_id, goods_id, buy_count)
             VALUES (?, ?, ?)
             ON CONFLICT(user_id, goods_id)
             DO UPDATE SET buy_count = buy_count + ?",
        )
        .bind(player_id)
        .bind(goods_id)
        .bind(quantity)
        .bind(quantity)
        .execute(pool)
        .await?;

        tracing::info!(
            "User {} purchased goods {} x{}",
            player_id,
            goods_id,
            quantity
        );
    }

    let (items, currencies, equips, heroes, power_items, insight_selectors) =
        parse_store_product(&goods.product);
    let items: Vec<(u32, i32)> = items
        .iter()
        .map(|(id, amt)| (*id, amt * quantity))
        .collect();
    let currencies: Vec<(i32, i32)> = currencies
        .iter()
        .map(|(id, amt)| (*id, amt * quantity))
        .collect();
    let equips: Vec<(u32, i32)> = equips
        .iter()
        .map(|(id, amt)| (*id, amt * quantity))
        .collect();
    let heroes: Vec<(u32, i32)> = heroes
        .iter()
        .map(|(id, amt)| (*id, amt * quantity))
        .collect();
    let power_items: Vec<(u32, i32)> = power_items
        .iter()
        .map(|(id, amt)| (*id, amt * quantity))
        .collect();
    let insight_selectors: Vec<(u32, i32)> = insight_selectors
        .iter()
        .map(|(id, amt)| (*id, amt * quantity))
        .collect();

    let (
        changed_item_ids,
        changed_currency_ids,
        changed_equip_ids,
        hero_dupe_items,
        hero_dupe_currencies,
    ) = {
        let ctx_guard = ctx.lock().await;
        let pool = &ctx_guard.state.db;

        let mut item_ids = if !items.is_empty() {
            item.create_items(&items).await?
        } else {
            vec![]
        };

        let mut currency_ids = if !currencies.is_empty() {
            currency.create_currencies(&currencies).await?
        } else {
            vec![]
        };

        let equip_ids = if !equips.is_empty() {
            database::db::game::equipment::add_equipments(
                pool,
                player_id,
                &equips
                    .iter()
                    .map(|(id, count)| (*id as i32, *count))
                    .collect::<Vec<_>>(),
            )
            .await?
        } else {
            Vec::new()
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

        let mut hero_dupe_items = Vec::new();
        let mut hero_dupe_currencies = Vec::new();

        for (hero_id, _count) in &heroes {
            let hero_id = *hero_id as i32;

            if hero.has_hero(hero_id).await? {
                let duplicate_count = hero.add_hero_duplicate(hero_id).await?;

                let (dupe_items, dupe_currencies) =
                    crate::state::grant_dupe_rewards(hero_id, duplicate_count).await?;

                hero_dupe_items.extend(dupe_items);
                hero_dupe_currencies.extend(dupe_currencies);

                tracing::info!(
                    "User {} already has hero {}, granted dupe rewards (duplicate #{})",
                    player_id,
                    hero_id,
                    duplicate_count
                );
            } else {
                hero.create_hero(hero_id).await?;
                tracing::info!("User {} received new hero {}", player_id, hero_id);
            }
        }

        if !hero_dupe_items.is_empty() {
            let dupe_item_ids = item.create_items(&hero_dupe_items).await?;
            item_ids.extend(dupe_item_ids);
        }
        if !hero_dupe_currencies.is_empty() {
            let dupe_currency_ids = currency.create_currencies(&hero_dupe_currencies).await?;
            currency_ids.extend(dupe_currency_ids);
        }

        (
            item_ids,
            currency_ids,
            equip_ids,
            hero_dupe_items,
            hero_dupe_currencies,
        )
    };

    let mut material_changes = Vec::new();

    for (item_id, amount) in &items {
        material_changes.push((1, *item_id, *amount));
    }
    for (item_id, amount) in &hero_dupe_items {
        material_changes.push((1, *item_id, *amount));
    }
    for (currency_id, amount) in &currencies {
        material_changes.push((2, *currency_id as u32, *amount));
    }
    for (currency_id, amount) in &hero_dupe_currencies {
        material_changes.push((2, *currency_id as u32, *amount));
    }
    for (equip_id, amount) in &equips {
        material_changes.push((9, *equip_id, *amount));
    }
    for (power_item_id, amount) in &power_items {
        material_changes.push((10, *power_item_id, *amount));
    }
    for (insight_selector_id, amount) in &insight_selectors {
        material_changes.push((24, *insight_selector_id, *amount));
    }
    if hero_dupe_items.is_empty() && hero_dupe_currencies.is_empty() {
        for (hero_id, amount) in &heroes {
            material_changes.push((4, *hero_id, *amount));
        }
    }

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(
                CmdId::BuyGoodsCmd,
                BuyGoodsReply {
                    store_id,
                    goods_id,
                    num,
                    select_cost,
                },
                0,
                req.up_tag,
            )
            .await?;
    }

    let mut all_changed_items: Vec<u32> =
        changed_item_ids.into_iter().map(|id| id as u32).collect();

    all_changed_items.extend(cost_items.iter().map(|(id, _)| *id as u32));

    if !all_changed_items.is_empty() {
        push::send_item_change_push(ctx.clone(), player_id, all_changed_items, vec![], vec![])
            .await?;
    }

    if !changed_currency_ids.is_empty() || !cost_currencies.is_empty() {
        let mut all_changed_currencies = changed_currency_ids;
        all_changed_currencies.extend(cost_currencies.iter().map(|(id, _)| (*id, 0)));
        push::send_currency_change_push(ctx.clone(), player_id, all_changed_currencies).await?;
    }

    if !changed_equip_ids.is_empty() {
        push::send_equip_update_push_by_uid(ctx.clone(), player_id, &changed_equip_ids).await?;
    }

    if !material_changes.is_empty() {
        push::send_material_change_push(ctx.clone(), material_changes, Some(27)).await?;
    }
    tracing::info!("Successfully completed purchase for user {}", player_id);
    Ok(())
}
