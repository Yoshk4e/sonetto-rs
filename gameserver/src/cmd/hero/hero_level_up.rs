use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::utils::push;
use database::models::game::{
    currencies::UserCurrencyModel,
    heros::{HeroModel, UserHeroModel},
};
use prost::Message;
use sonettobuf::{CmdId, HeroLevelUpReply, HeroLevelUpRequest};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub async fn on_hero_level_up(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroLevelUpRequest::decode(&req.data[..])?;
    tracing::info!("Received HeroLevelUpRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let expect_level = request.expect_level.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(player_id, pool.clone());
    let currency = UserCurrencyModel::new(player_id, pool.clone());
    let hero_data = hero.get(hero_id).await?;

    if expect_level == hero_data.record.level {
        tracing::info!("Hero {} already at level {}", hero_id, expect_level);

        let reply = HeroLevelUpReply {
            hero_id: Some(hero_id),
            new_level: Some(expect_level),
        };

        let level_push = sonettobuf::HeroLevelUpUpdatePush {
            hero_id: Some(hero_id),
            new_level: Some(expect_level),
            new_rank: Some(hero_data.record.rank),
        };

        let hero_info: sonettobuf::HeroInfo = hero_data.into();
        let hero_push = sonettobuf::HeroUpdatePush {
            hero_updates: vec![hero_info],
        };

        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(CmdId::HeroLevelUpUpdatePushCmd, level_push)
            .await?;
        ctx_guard
            .send_push(CmdId::HeroHeroUpdatePushCmd, hero_push)
            .await?;
        ctx_guard
            .send_reply(CmdId::HeroLevelUpCmd, reply, 0, req.up_tag)
            .await?;

        return Ok(());
    }

    if expect_level < hero_data.record.level {
        tracing::warn!(
            "Invalid level up: expect_level {} < current level {}",
            expect_level,
            hero_data.record.level
        );
        return Err(AppError::InvalidRequest);
    }

    if expect_level > 180 {
        tracing::warn!("Invalid level up: expect_level {} > max 180", expect_level);
        return Err(AppError::InvalidRequest);
    }

    let game_data = data::exceldb::get();

    let character = game_data
        .character
        .iter()
        .find(|c| c.id == hero_id)
        .ok_or_else(|| {
            tracing::error!("Character {} not found in game data", hero_id);
            AppError::InvalidRequest
        })?;

    let hero_rare = character.rare;
    let old_level = hero_data.record.level;
    let new_rank = hero_data.record.rank;

    let mut total_costs: HashMap<i32, i32> = HashMap::new();

    for level in (hero_data.record.level + 1)..=expect_level {
        let cost_entry = game_data
            .character_cosume
            .iter()
            .find(|c| c.level == level && c.rare == hero_rare)
            .ok_or_else(|| {
                tracing::error!(
                    "Cost entry not found for level {} rare {} (hero {})",
                    level,
                    hero_rare,
                    hero_id
                );
                AppError::InvalidRequest
            })?;

        if cost_entry.cosume.is_empty() {
            continue;
        }

        for cost_part in cost_entry.cosume.split('|') {
            let parts: Vec<&str> = cost_part.split('#').collect();
            if parts.len() >= 3 && parts[0] == "2" {
                let currency_id: i32 = parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                let amount: i32 = parts[2].parse().map_err(|_| AppError::InvalidRequest)?;

                *total_costs.entry(currency_id).or_insert(0) += amount;
            }
        }
    }

    for (currency_id, amount) in &total_costs {
        let current = currency
            .get_currency(*currency_id)
            .await?
            .map(|c| c.quantity)
            .unwrap_or(0);

        if current < *amount {
            tracing::info!(
                "User {} insufficient currency {} for level up (has {}, needs {})",
                player_id,
                currency_id,
                current,
                amount
            );

            push::send_currency_change_push(ctx.clone(), player_id, vec![(*currency_id, 0)])
                .await?;

            let data = HeroLevelUpReply {
                hero_id: Some(hero_id),
                new_level: Some(hero_data.record.level),
            };

            let mut ctx_guard = ctx.lock().await;
            ctx_guard
                .send_reply(CmdId::HeroLevelUpCmd, data, 0, req.up_tag)
                .await?;

            return Ok(());
        }
    }

    for (currency_id, amount) in &total_costs {
        currency.remove_currency(*currency_id, *amount).await?;
    }

    let level_stats = game_data
        .character_level
        .iter()
        .filter(|l| l.hero_id == hero_id && l.level <= expect_level)
        .max_by_key(|l| l.level)
        .ok_or_else(|| {
            tracing::error!(
                "No level stats found for hero {} up to level {}",
                hero_id,
                expect_level
            );
            AppError::InvalidRequest
        })?;

    hero.level_up(hero_id, expect_level, level_stats).await?;

    tracing::info!(
        "User {} leveled hero {} from {} to {} using stats from milestone level {} (costs: {:?})",
        player_id,
        hero_id,
        old_level,
        expect_level,
        level_stats.level,
        total_costs
    );

    let consumed: Vec<(i32, i32)> = total_costs.into_iter().collect();

    if !consumed.is_empty() {
        push::send_currency_change_push(
            ctx.clone(),
            player_id,
            consumed.iter().map(|(id, _)| (*id, 0)).collect(),
        )
        .await?;
    }

    let reply = HeroLevelUpReply {
        hero_id: Some(hero_id),
        new_level: Some(expect_level),
    };

    {
        let mut ctx_guard = ctx.lock().await;

        let level_push = sonettobuf::HeroLevelUpUpdatePush {
            hero_id: Some(hero_id),
            new_level: Some(expect_level),
            new_rank: Some(new_rank),
        };
        ctx_guard
            .send_push(CmdId::HeroLevelUpUpdatePushCmd, level_push)
            .await?;

        drop(ctx_guard);

        let updated_hero_data = hero.get(hero_id).await?;
        let updated_hero_info: sonettobuf::HeroInfo = updated_hero_data.into();

        let hero_push = sonettobuf::HeroUpdatePush {
            hero_updates: vec![updated_hero_info],
        };

        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(CmdId::HeroHeroUpdatePushCmd, hero_push)
            .await?;

        ctx_guard
            .send_reply(CmdId::HeroLevelUpCmd, reply, 0, req.up_tag)
            .await?;

        tracing::info!(
            "Sent HeroLevelUpUpdatePush and HeroUpdatePush for hero {} to level {}",
            hero_id,
            expect_level
        );
    }

    Ok(())
}
