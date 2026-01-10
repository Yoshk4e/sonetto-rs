use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::utils::push::{send_dungeon_update_push, send_end_dungeon_push, send_red_dot_push};

use crate::send_push;
use crate::state::{
    BattleSimulator, ConnectionContext, generate_dungeon_rewards, send_end_fight_push,
};
use database::db::game::dungeons::{
    get_user_dungeon, should_update_dungeon_record, update_dungeon_progress,
};
use database::db::game::{
    battle::save_round_operations, dungeons::save_dungeon_record, equipment::build_equip_records,
};
use prost::Message;
use sonettobuf::{BeginRoundReply, BeginRoundRequest, CmdId, InstructionDungeonInfoPush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_begin_round(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = BeginRoundRequest::decode(&req.data[..])?;

    tracing::info!(
        "BeginRound: {} operations, auto={}",
        request.opers.len(),
        request.auto_oper.unwrap_or(false)
    );

    let (
        fight,
        current_deck,
        fight_group,
        chapter_id,
        episode_id,
        is_replay,
        battle_id,
        round_num,
        multiplication,
    ) = {
        let ctx_guard = ctx.lock().await;
        let battle = ctx_guard
            .active_battle
            .as_ref()
            .ok_or(AppError::InvalidRequest)?;

        (
            battle.fight.clone().ok_or(AppError::InvalidRequest)?,
            battle.current_deck.clone(),
            battle.fight_group.clone(),
            battle.chapter_id,
            battle.episode_id,
            battle.is_replay.unwrap_or(false),
            battle.fight_id.unwrap_or_default(),
            battle.current_round,
            battle.multiplication.unwrap_or(1),
        )
    };

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        (
            ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?,
            ctx_guard.state.db.clone(),
        )
    };

    let mut simulator = BattleSimulator::new(fight);
    let mut round = simulator
        .process_round(request.opers.clone(), current_deck)
        .await?;

    // Auto-complete battle for now
    round.is_finish = Some(true);
    let record_round = round.cur_round.unwrap_or(1);

    tracing::info!(
        "Round result: {} steps, {} cards, round={}, finish={}",
        round.fight_step.len(),
        round.team_a_cards1.len(),
        record_round,
        round.is_finish.unwrap_or(false)
    );

    let reply = BeginRoundReply {
        round: Some(round.clone()),
    };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::BeginRoundCmd, reply, 0, req.up_tag)
            .await?;
    }

    if !is_replay {
        // Save operations for replay
        save_round_operations(
            &pool,
            player_id,
            episode_id,
            battle_id,
            round_num,
            vec![], // TODO: Extract cloth_skill_opers from request
            request.opers,
        )
        .await?;

        // Update player's dungeon progress
        let stars_earned = 2; // TODO: Calculate based on performance
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
            "Battle completed: episode={}, round={}, record_saved={}",
            episode_id,
            record_round,
            should_save_record
        );
    } else {
        tracing::info!(
            "Replay completed: episode={}, round={}",
            episode_id,
            record_round
        );
    }

    send_end_fight_push(
        ctx.clone(),
        battle_id,
        1, // Win
        fight_group.clone().unwrap_or_default(),
        vec![],     // TODO: Actual battle stats
        vec![],     // No defender stats
        !is_replay, // is_record: only record real battles
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
        chapter_type, // From chapter Excel data
        2,            // TODO: Calculate today's chapter completions
        2,            // TODO: Calculate today's chapter attempts
    )
    .await?;

    // Generate rewards based on episode data

    let is_first_clear = updated_dungeon.challenge_count == 1;

    let rewards = generate_dungeon_rewards(episode_id, is_first_clear, multiplication);

    // Combine rewards for push
    let mut all_rewards = rewards.normal_bonus.clone();
    all_rewards.extend(rewards.first_bonus);
    all_rewards.extend(rewards.free_bonus);

    send_end_dungeon_push(ctx.clone(), chapter_id, episode_id, all_rewards).await?;

    send_red_dot_push(Arc::clone(&ctx), player_id, Some(vec![1027, 1047])).await?;

    Ok(())
}
