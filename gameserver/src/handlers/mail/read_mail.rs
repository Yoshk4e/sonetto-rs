use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::inventory::{add_currencies, add_items};
use crate::util::push;
use database::models::game::heros::UserHeroModel;
use prost::Message;
use sonettobuf::{CmdId, ReadMailReply, ReadMailRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_read_mail(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = ReadMailRequest::decode(&req.data[..])?;
    tracing::info!("Received ReadMailRequest: {:?}", request);

    let incr_id = request.incr_id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = conn.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(player_id, pool.clone());

    let (user_id, attachment, changed_items, changed_currencies, changed_equips, new_heroes) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let mail: (String, i32) = sqlx::query_as(
            "SELECT attachment, state FROM user_mails WHERE incr_id = ? AND user_id = ?",
        )
        .bind(incr_id as i64)
        .bind(player_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::InvalidRequest)?;

        let (attachment, current_state) = mail;

        if current_state == 1 {
            tracing::info!("Mail {} already claimed by user {}", incr_id, player_id);
            drop(conn);

            let reply = ReadMailReply {
                incr_id: Some(incr_id),
            };

            let mut conn = ctx.lock().await;
            conn.send_reply(CmdId::ReadMailCmd, reply, 0, req.up_tag)
                .await?;

            return Ok(());
        }

        let (items, currencies, equips, heroes, power_items, insight_selectors) =
            crate::state::parse_store_product(&attachment);

        let item_ids = if !items.is_empty() {
            add_items(pool, player_id, &items).await?
        } else {
            vec![]
        };

        let currency_ids = if !currencies.is_empty() {
            add_currencies(pool, player_id, &currencies).await?
        } else {
            vec![]
        };

        let equip_uids: Vec<i64> = if !equips.is_empty() {
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
            database::db::game::items::add_power_items(
                pool,
                player_id,
                &power_items
                    .iter()
                    .map(|(id, count)| (*id as i32, *count))
                    .collect::<Vec<_>>(),
            )
            .await?;
        }

        if !insight_selectors.is_empty() {
            add_items(pool, player_id, &insight_selectors).await?;
        }

        let mut new_heroes = Vec::new();

        for (hero_id, _count) in &heroes {
            let hero_id = *hero_id as i32;

            if hero.has_hero(hero_id).await? {
                let duplicate_count = hero.add_hero_duplicate(hero_id).await?;

                tracing::info!(
                    "User {} already has hero {}, granted dupe rewards (duplicate #{})",
                    player_id,
                    hero_id,
                    duplicate_count
                );
            } else {
                hero.create_hero(hero_id).await?;
                new_heroes.push(hero_id);
                tracing::info!("User {} received new hero {} from mail", player_id, hero_id);
            }
        }

        sqlx::query("UPDATE user_mails SET state = 1 WHERE incr_id = ? AND user_id = ?")
            .bind(incr_id as i64)
            .bind(player_id)
            .execute(pool)
            .await?;

        let now = common::time::ServerTime::now_ms();
        sqlx::query(
            "INSERT INTO user_mail_history
             (user_id, mail_incr_id, mail_id, attachment, action, action_time, state_at_action)
             SELECT user_id, incr_id, mail_id, attachment, 'claimed', ?, 1
             FROM user_mails
             WHERE incr_id = ?",
        )
        .bind(now)
        .bind(incr_id as i64)
        .execute(pool)
        .await?;

        tracing::info!(
            "User {} claimed mail {} rewards: {} items, {} currencies, {} equips, {} heroes, {} power items, {} insight selectors",
            player_id,
            incr_id,
            items.len(),
            currencies.len(),
            equips.len(),
            heroes.len(),
            power_items.len(),
            insight_selectors.len()
        );

        (
            player_id,
            attachment,
            item_ids,
            currency_ids,
            equip_uids,
            new_heroes,
        )
    };

    if !new_heroes.is_empty() {
        let conn = ctx.lock().await;
        let mut hero_infos = Vec::new();

        for hero_id in new_heroes {
            if let Ok(heros) = hero.get_hero(hero_id).await {
                hero_infos.push(heros.into());
            }
        }
        drop(conn);

        if !hero_infos.is_empty() {
            let hero_push = sonettobuf::HeroUpdatePush {
                hero_updates: hero_infos,
            };
            let mut conn = ctx.lock().await;
            conn.notify(CmdId::HeroHeroUpdatePushCmd, hero_push).await?;
        }
    }

    let reply = ReadMailReply {
        incr_id: Some(incr_id),
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::ReadMailCmd, reply, 0, req.up_tag)
            .await?;
    }

    if !changed_items.is_empty() {
        push::send_item_change_push(ctx.clone(), user_id, changed_items, vec![], vec![]).await?;
    }

    if !changed_currencies.is_empty() {
        push::send_currency_change_push(ctx.clone(), user_id, changed_currencies).await?;
    }

    if !changed_equips.is_empty() {
        push::send_equip_update_push_by_uid(ctx.clone(), user_id, &changed_equips).await?;
    }

    let mut material_changes = Vec::new();
    let (items, currencies, equips, heroes_parsed, power_items_parsed, insight_selectors_parsed) =
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
    for (hero_id, amount) in heroes_parsed {
        material_changes.push((4, hero_id, amount));
    }
    for (power_item_id, amount) in power_items_parsed {
        material_changes.push((10, power_item_id, amount));
    }
    for (insight_selector_id, amount) in insight_selectors_parsed {
        material_changes.push((24, insight_selector_id, amount));
    }

    if !material_changes.is_empty() {
        push::send_material_change_push(ctx.clone(), material_changes, Some(10)).await?;
    }

    Ok(())
}
