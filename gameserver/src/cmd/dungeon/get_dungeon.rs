use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::dungeons;
use sonettobuf::{CmdId, DungeonInfosPush, GetDungeonReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_dungeon(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let player_id = {
        let ctx_guard = ctx.lock().await;
        ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?
    };

    let (
        last_groups,
        maps,
        elements,
        reward_points,
        equip_sp,
        chapter_nums,
        finished_elements,
        finished_puzzles,
    ) = {
        let ctx_guard = ctx.lock().await;

        tokio::try_join!(
            dungeons::get_dungeon_last_hero_groups(&ctx_guard.state.db, player_id),
            dungeons::get_unlocked_maps(&ctx_guard.state.db, player_id),
            dungeons::get_elements(&ctx_guard.state.db, player_id),
            dungeons::get_reward_points(&ctx_guard.state.db, player_id),
            dungeons::get_equip_sp_chapters(&ctx_guard.state.db, player_id),
            dungeons::get_chapter_type_nums(&ctx_guard.state.db, player_id),
            dungeons::get_finished_elements(&ctx_guard.state.db, player_id),
            dungeons::get_finished_puzzles(&ctx_guard.state.db, player_id),
        )?
    };

    let reply = GetDungeonReply {
        dungeon_info_list: Vec::new(),
        last_hero_group: last_groups.into_iter().map(Into::into).collect(),
        map_ids: maps,
        elements,
        reward_point_info: reward_points.into_iter().map(Into::into).collect(),
        equip_sp_chapters: equip_sp,
        chapter_type_nums: chapter_nums.into_iter().map(Into::into).collect(),
        finish_elements: finished_elements,
        finish_puzzles: finished_puzzles,
        dungeon_info_size: None,
    };

    send_dungeon_info_pushes(ctx.clone(), player_id).await?;

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::GetDungeonCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}

async fn send_dungeon_info_pushes(
    ctx: Arc<Mutex<ConnectionContext>>,
    user_id: i64,
) -> Result<(), AppError> {
    let dungeon_chunks = {
        let ctx_guard = ctx.lock().await;
        database::db::game::dungeons::get_user_dungeons_chunked(&ctx_guard.state.db, user_id)
            .await?
    };

    tracing::info!(
        "Sending {} dungeon push chunks for user {}",
        dungeon_chunks.len(),
        user_id
    );

    for (i, chunk) in dungeon_chunks.into_iter().enumerate() {
        let push = DungeonInfosPush {
            dungeon_infos: chunk.into_iter().map(Into::into).collect(),
        };

        {
            let mut ctx_guard = ctx.lock().await;
            ctx_guard
                .send_push(CmdId::DungeonInfosPushCmd, push)
                .await?;
        }

        tracing::debug!("Sent dungeon push chunk {} for user {}", i + 1, user_id);
    }

    Ok(())
}
