use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::push;
use database::{
    models::game::heros::{HeroModel, UserHeroModel},
    models::game::{currencies::UserCurrencyModel, items::UserItemModel},
};
use prost::Message;
use sonettobuf::{CmdId, HeroRankUpReply, HeroRankUpRequest, HeroUpdatePush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_rank_up(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroRankUpRequest::decode(&req.data[..])?;
    tracing::info!("Received HeroRankUpRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        (player_id, pool.clone())
    };

    let hero = UserHeroModel::new(player_id, pool.clone());
    let item = UserItemModel::new(player_id, pool.clone());
    let currency = UserCurrencyModel::new(player_id, pool.clone());

    let (user_id, new_rank, consumed_items, consumed_currencies) = {
        let conn = ctx.lock().await;

        let hero_data = hero.get(hero_id).await?;
        let hero_info: sonettobuf::HeroInfo = hero_data.clone().into();

        let current_rank = hero_data.record.rank;
        let target_rank = current_rank + 1;

        let game_data = data::exceldb::get();

        let rank_data = game_data
            .character_rank
            .iter()
            .find(|r| r.hero_id == hero_id && r.rank == target_rank);

        let rank_data = match rank_data {
            Some(r) => r,
            None => {
                tracing::info!(
                    "User {} hero {} already at max rank {}",
                    player_id,
                    hero_id,
                    current_rank
                );

                let reply = HeroRankUpReply {
                    hero_id: Some(hero_id),
                    new_rank: Some(current_rank),
                };

                drop(conn);

                let mut conn = ctx.lock().await;

                conn.notify(
                    CmdId::HeroHeroUpdatePushCmd,
                    HeroUpdatePush {
                        hero_updates: vec![hero_info],
                    },
                )
                .await?;
                conn.send_reply(CmdId::HeroRankUpCmd, reply, 0, req.up_tag)
                    .await?;

                return Ok(());
            }
        };

        if !rank_data.requirement.is_empty() {
            let req_parts: Vec<&str> = rank_data.requirement.split('#').collect();
            if req_parts.len() >= 2 && req_parts[0] == "1" {
                let required_level: i32 =
                    req_parts[1].parse().map_err(|_| AppError::InvalidRequest)?;

                if hero_data.record.level != required_level {
                    tracing::info!(
                        "Hero {} level {} does not match requirement {} for rank {} (retry)",
                        hero_id,
                        hero_data.record.level,
                        required_level,
                        target_rank
                    );

                    let reply = HeroRankUpReply {
                        hero_id: Some(hero_id),
                        new_rank: Some(current_rank),
                    };

                    drop(conn);

                    let mut conn = ctx.lock().await;

                    let updated_hero_data = hero.get(hero_id).await?;
                    let updated_hero_info: sonettobuf::HeroInfo = updated_hero_data.into();

                    conn.notify(
                        CmdId::HeroHeroUpdatePushCmd,
                        HeroUpdatePush {
                            hero_updates: vec![updated_hero_info],
                        },
                    )
                    .await?;
                    conn.send_reply(CmdId::HeroRankUpCmd, reply, 0, req.up_tag)
                        .await?;

                    return Ok(());
                }
            }
        }

        let mut cost_items = Vec::new();
        let mut cost_currencies = Vec::new();

        if !rank_data.consume.is_empty() {
            for cost_part in rank_data.consume.split('|') {
                let parts: Vec<&str> = cost_part.split('#').collect();
                if parts.len() >= 3 {
                    match parts[0] {
                        "1" => {
                            let item_id: u32 =
                                parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                            let amount: i32 =
                                parts[2].parse().map_err(|_| AppError::InvalidRequest)?;
                            cost_items.push((item_id, amount));
                        }
                        "2" => {
                            let currency_id: i32 =
                                parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                            let amount: i32 =
                                parts[2].parse().map_err(|_| AppError::InvalidRequest)?;
                            cost_currencies.push((currency_id, amount));
                        }
                        _ => {}
                    }
                }
            }
        }

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

                drop(conn);
                push::send_item_change_push(ctx.clone(), player_id, vec![*item_id], vec![], vec![])
                    .await?;

                let mut conn = ctx.lock().await;
                conn.send_reply(
                    CmdId::HeroRankUpCmd,
                    HeroRankUpReply {
                        hero_id: Some(hero_id),
                        new_rank: Some(current_rank),
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

                drop(conn);
                push::send_currency_change_push(ctx.clone(), player_id, vec![(*currency_id, 0)])
                    .await?;

                let mut conn = ctx.lock().await;
                conn.send_reply(
                    CmdId::HeroRankUpCmd,
                    HeroRankUpReply {
                        hero_id: Some(hero_id),
                        new_rank: Some(current_rank),
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

        hero.rank_up(hero_id, target_rank).await?;

        tracing::info!(
            "User {} ranked up hero {} from rank {} to {} (level reset to 1)",
            player_id,
            hero_id,
            current_rank,
            target_rank
        );

        hero.unlock_insight_skin(hero_id, target_rank).await?;

        (player_id, target_rank, cost_items, cost_currencies)
    };

    if !consumed_items.is_empty() {
        push::send_item_change_push(
            ctx.clone(),
            user_id,
            consumed_items.iter().map(|(id, _)| *id).collect(),
            vec![],
            vec![],
        )
        .await?;
    }

    if !consumed_currencies.is_empty() {
        push::send_currency_change_push(
            ctx.clone(),
            user_id,
            consumed_currencies.iter().map(|(id, _)| (*id, 0)).collect(),
        )
        .await?;
    }

    {
        let mut conn = ctx.lock().await;
        let updated_hero_data = hero.get(hero_id).await?;
        let updated_hero_info: sonettobuf::HeroInfo = updated_hero_data.into();

        conn.notify(
            CmdId::HeroHeroUpdatePushCmd,
            HeroUpdatePush {
                hero_updates: vec![updated_hero_info.into()],
            },
        )
        .await?;

        conn.send_reply(
            CmdId::HeroRankUpCmd,
            HeroRankUpReply {
                hero_id: Some(hero_id),
                new_rank: Some(new_rank),
            },
            0,
            req.up_tag,
        )
        .await?;
    }

    tracing::info!("Hero {} ranked up to {}", hero_id, new_rank);

    Ok(())
}
