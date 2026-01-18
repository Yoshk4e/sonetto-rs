use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::{
    currencies::UserCurrencyModel,
    heros::{HeroModel, UserHeroModel},
    items::UserItemModel,
};
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, UnlockTalentStyleReply, UnlockTalentStyleRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_unlock_talent_style(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UnlockTalentStyleRequest::decode(&req.data[..])?;
    tracing::info!("Received UnlockTalentStyleRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let style = request.style.ok_or(AppError::InvalidRequest)?;

    let (user_id, pool) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = conn.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(user_id, pool.clone());
    let item = UserItemModel::new(user_id, pool.clone());
    let currency = UserCurrencyModel::new(user_id, pool.clone());

    if hero.has_talent_style(hero_id, style).await? {
        tracing::info!(
            "User {} already owns style {} for hero {}",
            user_id,
            style,
            hero_id
        );

        let hero_data = hero.get(hero_id).await?;
        let hero_proto: sonettobuf::HeroInfo = hero_data.into();

        let mut conn = ctx.lock().await;
        conn.notify(
            CmdId::HeroHeroUpdatePushCmd,
            HeroUpdatePush {
                hero_updates: vec![hero_proto],
            },
        )
        .await?;
        conn.send_reply(
            CmdId::UnlockTalentStyleCmd,
            UnlockTalentStyleReply {
                hero_id: Some(hero_id),
                style: Some(style),
            },
            0,
            req.up_tag,
        )
        .await?;

        return Ok(());
    }

    let (cost_items, cost_currencies) = {
        let game_data = data::exceldb::get();
        let style_cost = game_data
            .talent_style_cost
            .iter()
            .find(|s| s.hero_id == hero_id && s.style_id == style)
            .ok_or(AppError::InvalidRequest)?;

        let mut items = Vec::new();
        let mut currencies = Vec::new();

        for cost_part in style_cost.consume.split('|') {
            let parts: Vec<&str> = cost_part.split('#').collect();
            if parts.len() >= 3 {
                match parts[0] {
                    "1" => {
                        let item_id: u32 =
                            parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                        let amount: i32 = parts[2].parse().map_err(|_| AppError::InvalidRequest)?;
                        items.push((item_id, amount));
                    }
                    "2" => {
                        let currency_id: i32 =
                            parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                        let amount: i32 = parts[2].parse().map_err(|_| AppError::InvalidRequest)?;
                        currencies.push((currency_id, amount));
                    }
                    _ => {}
                }
            }
        }

        (items, currencies)
    };

    for (item_id, amount) in &cost_items {
        let success = item.remove_item_quantity(*item_id, *amount).await?;
        if !success {
            tracing::info!("User {} insufficient item {}", user_id, item_id);

            crate::util::push::send_item_change_push(
                ctx.clone(),
                user_id,
                vec![*item_id],
                vec![],
                vec![],
            )
            .await?;

            let mut conn = ctx.lock().await;
            conn.send_reply(
                CmdId::UnlockTalentStyleCmd,
                UnlockTalentStyleReply {
                    hero_id: Some(hero_id),
                    style: Some(style),
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
            tracing::info!("User {} insufficient currency {}", user_id, currency_id);

            crate::util::push::send_currency_change_push(
                ctx.clone(),
                user_id,
                vec![(*currency_id, 0)],
            )
            .await?;

            let mut conn = ctx.lock().await;
            conn.send_reply(
                CmdId::UnlockTalentStyleCmd,
                UnlockTalentStyleReply {
                    hero_id: Some(hero_id),
                    style: Some(style),
                },
                0,
                req.up_tag,
            )
            .await?;

            return Ok(());
        }
    }

    for (currency_id, amount) in &cost_currencies {
        currency.remove_currency(*currency_id, *amount).await?;
    }

    hero.unlock_talent_style(hero_id, style).await?;

    let data = UnlockTalentStyleReply {
        hero_id: Some(hero_id),
        style: Some(style),
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

        conn.send_reply(CmdId::UnlockTalentStyleCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
