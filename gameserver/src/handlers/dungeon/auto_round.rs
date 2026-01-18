use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::util::push::{send_dungeon_update_push, send_end_dungeon_push, send_red_dot_push};

use crate::send_push;
use crate::state::{
    BattleSimulator, ConnectionContext, generate_auto_opers, generate_dungeon_rewards,
    send_end_fight_push,
};
use database::db::game::dungeons::{
    get_user_dungeon, should_update_dungeon_record, update_dungeon_progress,
};
use database::db::game::{
    battle::save_round_operations, dungeons::save_dungeon_record, equipment::build_equip_records,
};
use prost::Message;
use sonettobuf::{AutoRoundReply, AutoRoundRequest, CmdId, InstructionDungeonInfoPush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_auto_round(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = AutoRoundRequest::decode(&req.data[..])?;

    tracing::info!(
        "AutoRound request: client_opers: {:?}, client_opers_len={}, to_id={}",
        request.opers,
        request.opers.len(),
        request.to_id.unwrap_or(0)
    );

    let (
        current_deck,
        fight_group,
        chapter_id,
        episode_id,
        is_replay,
        battle_id,
        round_num,
        multiplication,
        ai_deck,
        fight_data_mgr,
    ) = {
        let conn = ctx.lock().await;
        let battle = conn
            .active_battle
            .as_ref()
            .ok_or(AppError::InvalidRequest)?;

        (
            battle.current_deck.clone(),
            battle.fight_group.clone(),
            battle.chapter_id,
            battle.episode_id,
            battle.is_replay.unwrap_or(false),
            battle.fight_id.unwrap_or_default(),
            battle.current_round,
            battle.multiplication.unwrap_or(1),
            battle.ai_deck.clone(),
            battle.fight_data_mgr.clone().unwrap_or_default(),
        )
    };

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };

    let auto_opers = generate_auto_opers(&current_deck);

    tracing::info!("AutoRound server selected {} ops", auto_opers.len());

    let mut simulator = BattleSimulator::new(fight_data_mgr);
    let mut round = simulator
        .process_round(auto_opers.clone(), current_deck, ai_deck)
        .await?;

    round.is_finish = Some(true);
    let record_round = round.cur_round.unwrap_or(1);

    tracing::info!(
        "AutoRound result: steps={}, cards={}, round={}, finished={}",
        round.fight_step.len(),
        round.team_a_cards1.len(),
        record_round,
        round.is_finish.unwrap_or(false)
    );

    let reply = AutoRoundReply {
        opers: auto_opers.clone(),
        to_id: request.to_id.or(Some(1)),
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::AutoRoundCmd, reply, 0, req.up_tag)
            .await?;
    }

    if !is_replay {
        save_round_operations(
            &pool,
            player_id,
            episode_id,
            battle_id,
            round_num,
            vec![], // cloth ops (future)
            auto_opers,
        )
        .await?;

        let stars_earned = 2; // TODO real calc
        update_dungeon_progress(&pool, player_id, chapter_id, episode_id, stars_earned).await?;

        let should_save_record =
            should_update_dungeon_record(&pool, player_id, episode_id, record_round, &fight_group)
                .await?;

        if should_save_record {
            let equips = build_equip_records(&pool, player_id, &fight_group).await?;
            save_dungeon_record(
                &pool,
                player_id,
                episode_id,
                record_round,
                &fight_group.clone().unwrap_or_default(),
                equips,
            )
            .await?;
        }

        tracing::info!(
            "Auto battle completed: episode={}, round={}, record_saved={}",
            episode_id,
            record_round,
            should_save_record
        );
    }

    send_end_fight_push(
        ctx.clone(),
        battle_id,
        1,
        fight_group.clone().unwrap_or_default(),
        vec![],
        vec![],
        !is_replay,
    )
    .await?;

    send_push!(
        ctx,
        CmdId::DungeonInstructionDungeonInfoPushCmd,
        InstructionDungeonInfoPush,
        "dungeon/instruction_dungeon_info.json"
    );

    let updated_dungeon = get_user_dungeon(&pool, player_id, chapter_id, episode_id).await?;

    let game_data = data::exceldb::get();
    let chapter_type = game_data
        .chapter
        .iter()
        .find(|c| c.id == chapter_id)
        .map(|c| c.r#type)
        .unwrap_or(6);

    send_dungeon_update_push(
        ctx.clone(),
        chapter_id,
        episode_id,
        updated_dungeon.star,
        updated_dungeon.challenge_count,
        updated_dungeon.has_record,
        chapter_type,
        2,
        2,
    )
    .await?;

    let is_first_clear = updated_dungeon.challenge_count == 1;
    let rewards = generate_dungeon_rewards(episode_id, is_first_clear, multiplication);

    let mut all_rewards = rewards.normal_bonus.clone();
    all_rewards.extend(rewards.first_bonus);
    all_rewards.extend(rewards.free_bonus);

    send_end_dungeon_push(ctx.clone(), chapter_id, episode_id, all_rewards).await?;
    send_red_dot_push(ctx.clone(), player_id, Some(vec![1027, 1047])).await?;

    Ok(())
}
