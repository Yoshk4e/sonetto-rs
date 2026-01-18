use super::process_item_use;
use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::{ConnectionContext, grant_dupe_rewards};
use crate::util::push;

use database::models::game::{
    currencies::UserCurrencyModel, heros::UserHeroModel, items::UserItemModel,
};
use prost::Message;
use sonettobuf::{CmdId, UseItemReply, UseItemRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_use_item(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UseItemRequest::decode(&req.data[..])?;
    tracing::info!("Received use item request: {:?}", request);

    let user_id = ctx.lock().await.player_id.ok_or(AppError::NotLoggedIn)?;
    let pool = ctx.lock().await.state.db.clone();

    let item = UserItemModel::new(user_id, pool.clone());
    let currency = UserCurrencyModel::new(user_id, pool.clone());
    let hero = UserHeroModel::new(user_id, pool.clone());

    let mut all_items = Vec::new();
    let mut all_currencies = Vec::new();
    let mut all_equips = Vec::new();
    let mut consumed_items = Vec::new();

    for entry in &request.entry {
        let material_id = entry.material_id.ok_or(AppError::InvalidRequest)?;
        let quantity = entry.quantity.unwrap_or(1);

        if !item.remove_item_quantity(material_id, quantity).await? {
            return Err(AppError::InsufficientItems);
        }
        consumed_items.push(material_id);

        let is_hero_item = matches!(material_id, 252701 | 550001 | 520010);
        let is_hero_selector = matches!(material_id, 481022 | 481004);

        let (item_rewards, currency_rewards) = if is_hero_selector && request.target_id.is_some() {
            let game_data = data::exceldb::get();
            if let Some(cfg) = game_data.item.get(material_id as i32) {
                let hero_ids: Vec<i32> = cfg
                    .effect
                    .split('|')
                    .filter_map(|segment| {
                        let parts: Vec<&str> = segment.split('#').collect();
                        if parts.len() == 3 && parts[0] == "4" {
                            parts[1].parse::<i32>().ok()
                        } else {
                            None
                        }
                    })
                    .collect();

                let target_idx = request.target_id.unwrap() as usize;
                if let Some(&hero_id) = hero_ids.get(target_idx) {
                    let duplicate_count = if hero.has_hero(hero_id).await? {
                        hero.add_hero_duplicate(hero_id).await?
                    } else {
                        hero.create_hero(hero_id).await?;
                        0
                    };
                    grant_dupe_rewards(hero_id, duplicate_count).await?
                } else {
                    tracing::warn!("Invalid hero index {} for item {}", target_idx, material_id);
                    (vec![], vec![])
                }
            } else {
                (vec![], vec![])
            }
        } else if is_hero_item && request.target_id.is_some() {
            let hero_id = request.target_id.unwrap() as i32;
            let duplicate_count = if hero.has_hero(hero_id).await? {
                hero.add_hero_duplicate(hero_id).await?
            } else {
                hero.create_hero(hero_id).await?;
                0
            };
            grant_dupe_rewards(hero_id, duplicate_count).await?
        } else {
            process_item_use(material_id, quantity, request.target_id)
        };

        for (item_id, amount) in item_rewards {
            if let Some(existing) = all_items.iter_mut().find(|(id, _)| *id == item_id) {
                existing.1 += amount;
            } else {
                all_items.push((item_id, amount));
            }
        }

        for (currency_id, amount) in currency_rewards {
            if let Some(existing) = all_currencies.iter_mut().find(|(id, _)| *id == currency_id) {
                existing.1 += amount;
            } else {
                all_currencies.push((currency_id, amount));
            }
        }
    }

    let game_data = data::exceldb::get();
    let mut final_items = Vec::new();

    for (item_id, amount) in all_items {
        if game_data.equip.get(item_id as i32).is_some() {
            all_equips.push((item_id, amount));
        } else {
            final_items.push((item_id, amount));
        }
    }

    let mut material_changes = Vec::new();
    for (item_id, amount) in &final_items {
        material_changes.push((1, *item_id, *amount));
    }
    for (currency_id, amount) in &all_currencies {
        material_changes.push((2, *currency_id as u32, *amount));
    }
    for (equip_id, amount) in &all_equips {
        material_changes.push((9, *equip_id, *amount));
    }

    tracing::info!(
        "Total rewards: {} items, {} currencies, {} equips",
        final_items.len(),
        all_currencies.len(),
        all_equips.len()
    );

    let reward_items = if !final_items.is_empty() {
        item.create_items(&final_items).await?
    } else {
        vec![]
    };

    let reward_currencies = if !all_currencies.is_empty() {
        currency.create_currencies(&all_currencies).await?
    } else {
        vec![]
    };

    let reward_uids: Vec<i64> = if !all_equips.is_empty() {
        database::db::game::equipment::add_equipments(
            &pool,
            user_id,
            &all_equips
                .iter()
                .map(|(id, count)| (*id as i32, *count))
                .collect::<Vec<_>>(),
        )
        .await?
    } else {
        Vec::new()
    };

    ctx.lock()
        .await
        .send_reply(
            CmdId::UseItemCmd,
            UseItemReply {
                entry: request.entry,
                target_id: request.target_id,
            },
            0,
            req.up_tag,
        )
        .await?;

    consumed_items.extend(reward_items.iter().map(|&id| id as u32));
    if !consumed_items.is_empty() {
        push::send_item_change_push(ctx.clone(), user_id, consumed_items, vec![], vec![]).await?;
    }

    if !reward_currencies.is_empty() {
        push::send_currency_change_push(ctx.clone(), user_id, reward_currencies).await?;
    }

    if !reward_uids.is_empty() {
        push::send_equip_update_push_by_uid(ctx.clone(), user_id, &reward_uids).await?;
    }

    if !material_changes.is_empty() {
        push::send_material_change_push(ctx.clone(), material_changes, Some(26)).await?;
    }

    Ok(())
}
