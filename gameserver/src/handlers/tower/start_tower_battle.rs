use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::{
    ActiveBattle, BattleContext, ConnectionContext, create_battle, default_max_ap,
    generate_initial_deck,
};
use data::exceldb;
use prost::Message;
use sonettobuf::{
    CmdId, DungeonUpdatePush, StartDungeonReply, StartTowerBattleReply, StartTowerBattleRequest,
    UserDungeon,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_start_tower_battle(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = StartTowerBattleRequest::decode(&req.data[..])?;

    let start_req = request
        .start_dungeon_request
        .ok_or(AppError::InvalidRequest)?;
    let fight_group = start_req.fight_group.ok_or(AppError::InvalidRequest)?;

    let dungeon_type = request.r#type.ok_or(AppError::InvalidRequest)?;
    let tower_id = request.tower_id.ok_or(AppError::InvalidRequest)?;
    let layer_id = request.layer_id.ok_or(AppError::InvalidRequest)?;
    let difficulty = request.difficulty.ok_or(AppError::InvalidRequest)?;
    let talent_plan_id = request.talent_plan_id.unwrap_or(0);

    let chapter_id = start_req.chapter_id.unwrap_or(0);
    let episode_id = start_req.episode_id.unwrap_or(0);

    tracing::info!(
        "Start tower battle: type={}, tower={}, layer={}, episode={}, diff={}, talent={}",
        dungeon_type,
        tower_id,
        layer_id,
        episode_id,
        difficulty,
        talent_plan_id
    );

    tracing::info!(
        "Fight group: heroes={:?}, cloth={}, assist_boss={}",
        fight_group.hero_list,
        fight_group.cloth_id.unwrap_or(1),
        fight_group.assist_boss_id.unwrap_or(0)
    );

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };

    let hero_count = fight_group.hero_list.iter().filter(|&&u| u != 0).count();

    let game_data = exceldb::get();
    let battle_id = game_data
        .episode
        .iter()
        .find(|e| e.id == episode_id)
        .ok_or(AppError::InvalidRequest)?
        .battle_id;

    let max_ap = default_max_ap(episode_id, hero_count);

    let battle_ctx = BattleContext {
        player_id,
        chapter_id,
        episode_id,
        battle_id,
        max_ap,
    };

    let card_push = generate_initial_deck(&pool, player_id, &fight_group, max_ap).await?;

    let card_deck = card_push.card_group.clone();

    let (modified_fight, initial_round, fight_data_mgr, ai_deck) =
        create_battle(&pool, battle_ctx, &fight_group, card_deck.clone()).await?;

    {
        let mut conn = ctx.lock().await;
        conn.active_battle = Some(ActiveBattle {
            tower_type: Some(dungeon_type),
            tower_id: Some(tower_id),
            layer_id: Some(layer_id),
            episode_id,
            chapter_id,
            difficulty: Some(difficulty),
            talent_plan_id: Some(talent_plan_id),
            fight: Some(modified_fight.clone()),
            current_round: 1,
            act_point: max_ap,
            power: 15,
            current_deck: card_deck,
            fight_group: Some(fight_group.clone()),
            is_replay: None,
            replay_episode_id: None,
            fight_id: Some(chrono::Utc::now().timestamp_millis()),
            multiplication: None,
            ai_deck,
            fight_data_mgr: Some(fight_data_mgr),
        });
    }

    {
        let mut conn = ctx.lock().await;
        conn.notify(CmdId::CardInfoPushCmd, card_push).await?;
    }

    let start_reply = StartTowerBattleReply {
        start_dungeon_reply: Some(StartDungeonReply {
            fight: Some(modified_fight),
            round: Some(initial_round),
        }),
        r#type: Some(dungeon_type),
        tower_id: Some(tower_id),
        layer_id: Some(layer_id),
        difficulty: Some(difficulty),
        talent_plan_id: Some(talent_plan_id),
    };

    let dungeon_push = DungeonUpdatePush {
        dungeon_info: Some(UserDungeon {
            chapter_id: Some(chapter_id),
            episode_id: Some(episode_id),
            star: Some(0),
            challenge_count: Some(0),
            has_record: Some(false),
            left_return_all_num: Some(1),
            today_pass_num: Some(0),
            today_total_num: Some(0),
        }),
        chapter_type_nums: vec![],
    };

    let mut conn = ctx.lock().await;

    conn.notify(CmdId::DungeonUpdatePushCmd, dungeon_push)
        .await?;

    conn.send_reply(CmdId::StartTowerBattleCmd, start_reply, 0, req.up_tag)
        .await?;

    Ok(())
}
