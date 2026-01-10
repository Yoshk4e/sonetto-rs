use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::models::game::{
    heros::{HeroModel, UserHeroModel},
    items::UserItemModel,
};
use prost::Message;
use sonettobuf::{CmdId, HeroTalentUpReply, HeroTalentUpRequest, HeroUpdatePush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_talent_up(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroTalentUpRequest::decode(&req.data[..])?;
    tracing::info!("Received HeroTalentUpRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = ctx_guard.state.db.clone();
        (player_id, pool)
    };

    let hero = UserHeroModel::new(player_id, pool.clone());
    let item = UserItemModel::new(player_id, pool.clone());

    let new_talent_id = {
        let ctx_guard = ctx.lock().await;

        let hero_data = hero.get(hero_id).await?;

        let current_talent = hero_data.record.talent;
        let new_talent = current_talent + 1;

        let game_data = data::exceldb::get();

        let talent_config = game_data
            .character_talent
            .iter()
            .find(|t| t.hero_id == hero_id && t.talent_id == new_talent);

        let talent_config = match talent_config {
            Some(t) => t,
            None => {
                tracing::info!("Hero {} already at max talent {}", hero_id, current_talent);

                let reply = HeroTalentUpReply {
                    hero_id: Some(hero_id),
                    talent_id: Some(current_talent),
                };

                drop(ctx_guard);

                let mut ctx_guard = ctx.lock().await;
                let hero_proto: sonettobuf::HeroInfo = hero_data.into();
                ctx_guard
                    .send_push(
                        CmdId::HeroHeroUpdatePushCmd,
                        HeroUpdatePush {
                            hero_updates: vec![hero_proto],
                        },
                    )
                    .await?;
                ctx_guard
                    .send_reply(CmdId::HeroTalentUpCmd, reply, 0, req.up_tag)
                    .await?;

                return Ok(());
            }
        };

        if hero_data.record.rank < talent_config.requirement {
            tracing::info!(
                "Hero {} rank {} does not meet talent {} requirement (needs rank {})",
                hero_id,
                hero_data.record.rank,
                new_talent,
                talent_config.requirement
            );

            let reply = HeroTalentUpReply {
                hero_id: Some(hero_id),
                talent_id: Some(current_talent),
            };

            drop(ctx_guard);

            let mut ctx_guard = ctx.lock().await;
            let hero_proto: sonettobuf::HeroInfo = hero_data.into();
            ctx_guard
                .send_push(
                    CmdId::HeroHeroUpdatePushCmd,
                    HeroUpdatePush {
                        hero_updates: vec![hero_proto],
                    },
                )
                .await?;
            ctx_guard
                .send_reply(CmdId::HeroTalentUpCmd, reply, 0, req.up_tag)
                .await?;

            return Ok(());
        }

        if !talent_config.consume.is_empty() {
            for cost_part in talent_config.consume.split('|') {
                let parts: Vec<&str> = cost_part.split('#').collect();
                if parts.len() >= 3 && parts[0] == "1" {
                    let item_id: u32 = parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                    let amount: i32 = parts[2].parse().map_err(|_| AppError::InvalidRequest)?;

                    let current = item
                        .get_item(item_id)
                        .await?
                        .map(|i| i.quantity)
                        .unwrap_or(0);

                    if current < amount {
                        tracing::info!(
                            "User {} insufficient item {} for talent up (has {}, needs {})",
                            player_id,
                            item_id,
                            current,
                            amount
                        );

                        drop(ctx_guard);

                        crate::utils::push::send_item_change_push(
                            ctx.clone(),
                            player_id,
                            vec![item_id],
                            vec![],
                            vec![],
                        )
                        .await?;

                        let mut ctx_guard = ctx.lock().await;
                        ctx_guard
                            .send_reply(
                                CmdId::HeroTalentUpCmd,
                                HeroTalentUpReply {
                                    hero_id: Some(hero_id),
                                    talent_id: Some(current_talent),
                                },
                                0,
                                req.up_tag,
                            )
                            .await?;

                        return Ok(());
                    }
                }
            }

            for cost_part in talent_config.consume.split('|') {
                let parts: Vec<&str> = cost_part.split('#').collect();
                if parts.len() >= 3 && parts[0] == "1" {
                    let item_id: u32 = parts[1].parse().unwrap();
                    let amount: i32 = parts[2].parse().unwrap();

                    item.remove_item_quantity(item_id, amount).await?;
                }
            }
        }

        hero.update_talent(hero_id, new_talent).await?;

        tracing::info!(
            "User {} upgraded hero {} talent from {} to {}",
            player_id,
            hero_id,
            current_talent,
            new_talent
        );

        new_talent
    };

    let data = HeroTalentUpReply {
        hero_id: Some(hero_id),
        talent_id: Some(new_talent_id),
    };

    {
        let mut ctx_guard = ctx.lock().await;

        let hero_data = hero.get(hero_id).await?;
        let hero_info: sonettobuf::HeroInfo = hero_data.into();

        ctx_guard
            .send_push(
                CmdId::HeroHeroUpdatePushCmd,
                HeroUpdatePush {
                    hero_updates: vec![hero_info.into()],
                },
            )
            .await?;

        ctx_guard
            .send_reply(CmdId::HeroTalentUpCmd, data, 0, req.up_tag)
            .await?;

        tracing::info!("Hero {} talent upgraded to {}", hero_id, new_talent_id);
    }

    Ok(())
}
