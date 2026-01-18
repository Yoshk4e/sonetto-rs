use crate::state::battle::manager::fight_data_mgr::FightDataMgr;

use anyhow::Result;
use sonettobuf::{CardInfo, Fight, FightRound};

pub async fn build_initial_round(
    fight: Fight,
    player_deck: Vec<CardInfo>,
    ai_deck: Vec<CardInfo>,
) -> Result<(FightRound, Fight, FightDataMgr)> {
    let mut fight_mgr = FightDataMgr::new(fight);

    let round = fight_mgr.build_initial_round(player_deck, ai_deck)?;

    let updated_fight = fight_mgr.get_fight_owned();

    for entity in fight_mgr.entity_mgr.get_team_entities(1) {
        tracing::warn!(
            "FINAL ENTITY: uid={} current_hp={} max_hp={}",
            entity.uid.unwrap_or(0),
            entity.current_hp.unwrap_or(0),
            entity.attr.as_ref().and_then(|a| a.hp).unwrap_or(0),
        );
    }

    Ok((round, updated_fight, fight_mgr))
}
