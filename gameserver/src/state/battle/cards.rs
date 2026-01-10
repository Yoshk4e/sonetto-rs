use rand::{seq::SliceRandom, thread_rng};
use sqlx::SqlitePool;

use crate::error::AppError;
use data::exceldb;
use database::models::game::heros::{HeroModel, UserHeroModel};
use sonettobuf::{CardInfo, CardInfoPush, FightGroup};

// Core deck generation
pub async fn generate_card_deck(
    pool: &SqlitePool,
    user_id: i64,
    fight_group: &FightGroup,
    max_cards: usize,
) -> Result<Vec<CardInfo>, AppError> {
    let active_heroes: Vec<i64> = fight_group
        .hero_list
        .iter()
        .copied()
        .filter(|&u| u != 0)
        .collect();

    let candidates = build_candidate_pool(pool, user_id, &active_heroes).await?;
    let deck = draw_cards_with_merge(candidates, max_cards);

    Ok(deck)
}

//  creates CardInfoPush for handlers
pub async fn generate_initial_deck(
    pool: &SqlitePool,
    user_id: i64,
    fight_group: &FightGroup,
    act_point: i32,
) -> Result<CardInfoPush, AppError> {
    let hero_count = fight_group.hero_list.iter().filter(|&&u| u != 0).count();
    let max_cards = compute_max_cards(hero_count);

    let deck = generate_card_deck(pool, user_id, fight_group, max_cards).await?;

    Ok(CardInfoPush {
        card_group: deck.clone(),
        deal_card_group: deck,
        act_point: Some(act_point),
        move_num: Some(0),
        before_cards: vec![],
        extra_move_act: Some(0),
        is_gm: Some(false),
    })
}

#[allow(dead_code)]
fn draw_cards_no_merge(candidates: Vec<CardInfo>, max_cards: usize) -> Vec<CardInfo> {
    let mut rng = thread_rng();
    let mut deck: Vec<CardInfo> = Vec::with_capacity(max_cards);

    for _ in 0..max_cards {
        let card = candidates
            .choose(&mut rng)
            .expect("candidate pool empty")
            .clone();
        deck.push(card);
    }

    deck
}

fn compute_max_cards(hero_count: usize) -> usize {
    (hero_count * 3).min(9)
}

async fn build_candidate_pool(
    pool: &SqlitePool,
    user_id: i64,
    hero_uids: &[i64],
) -> Result<Vec<CardInfo>, AppError> {
    let mut pool_cards = Vec::new();
    let game_data = exceldb::get();

    let hero = UserHeroModel::new(user_id, pool.clone());

    for &hero_uid in hero_uids {
        let hero_id = if hero_uid < 0 {
            // Trial hero - load from static data
            let trial_id = hero_uid.abs() as i32;
            let trial_data = game_data
                .hero_trial
                .iter()
                .find(|t| t.id == trial_id)
                .ok_or_else(|| {
                    tracing::error!("Trial hero {} not found in static data", trial_id);
                    AppError::InvalidRequest
                })?;

            tracing::info!(
                "Loading trial hero {}: hero_id={}, level={}, skin={}",
                trial_id,
                trial_data.hero_id,
                trial_data.level,
                trial_data.skin
            );

            trial_data.hero_id
        } else {
            // Regular hero - load from database
            let hero = hero.get_uid(hero_uid as i32).await?;
            hero.record.hero_id
        };

        for skill_id in get_hero_skills(hero_id) {
            pool_cards.push(CardInfo {
                uid: Some(hero_uid),
                hero_id: Some(hero_id),
                skill_id: Some(skill_id),
                card_type: Some(0), // rank 1
                status: Some(0),
                temp_card: Some(hero_uid < 0), // Mark trial hero cards as temp
                enchants: vec![],
                target_uid: Some(0),
                energy: Some(0),
                extra_infos: vec![],
                area_red_or_blue: Some(0),
                heat_id: Some(0),
                card_effect: None,
                extra_info: None,
            });
        }
    }

    Ok(pool_cards)
}

fn draw_cards_with_merge(candidates: Vec<CardInfo>, max_cards: usize) -> Vec<CardInfo> {
    let mut rng = thread_rng();
    let mut deck: Vec<CardInfo> = Vec::with_capacity(max_cards);

    while deck.len() < max_cards {
        let card = candidates
            .choose(&mut rng)
            .expect("candidate pool empty")
            .clone();

        if let Some(last) = deck.last_mut() {
            // Check if same hero and same base skill (differ only in rank)
            if last.hero_id == card.hero_id
                && last.skill_id.unwrap_or(0) / 10 == card.skill_id.unwrap_or(0) / 10
                && last.card_type == card.card_type
            {
                let new_rank = last.card_type.unwrap_or(1) + 1;

                let game_data = exceldb::get();
                if let Some(upgraded_skill) = game_data.skill.iter().find(|s| {
                    s.hero_id == last.hero_id.unwrap_or(0)
                        && s.skill_rank == new_rank
                        && s.id / 10 == last.skill_id.unwrap_or(0) / 10
                }) {
                    last.card_type = Some(new_rank);
                    last.skill_id = Some(upgraded_skill.id);

                    tracing::debug!(
                        "Merged: skill {} -> {}, rank {}",
                        card.skill_id.unwrap_or(0),
                        upgraded_skill.id,
                        new_rank
                    );

                    continue;
                }
            }
        }

        deck.push(card);
    }

    deck
}

fn get_hero_skills(hero_id: i32) -> Vec<i32> {
    let game_data = exceldb::get();

    let character = game_data.character.iter().find(|c| c.id == hero_id);

    let Some(character) = character else {
        tracing::warn!("Character {} not found in character table", hero_id);
        return Vec::new();
    };

    // Parse skill string: "1#31240111#31240112#31240113|2#31240121#31240122#31240123"
    let mut skills = Vec::new();

    for skill_group in character.skill.split('|') {
        let parts: Vec<&str> = skill_group.split('#').collect();

        if parts.is_empty() {
            continue;
        }

        // Skip the first part (skill group number like "1" or "2")
        // Take only the first skill ID from each group (rank 1)
        if parts.len() > 1 {
            if let Ok(skill_id) = parts[1].parse::<i32>() {
                skills.push(skill_id);
            }
        }
    }

    if skills.is_empty() {
        tracing::warn!(
            "No skills parsed for hero {} from skill string: {}",
            hero_id,
            character.skill
        );
    } else {
        tracing::debug!("Hero {} skills: {:?}", hero_id, skills);
    }

    skills.sort_unstable();
    skills
}

pub fn default_max_ap(episode_id: i32, hero_count: usize) -> i32 {
    let game_data = exceldb::get();

    let battle_id = game_data
        .episode
        .iter()
        .find(|t| t.id == episode_id)
        .map(|t| t.battle_id)
        .unwrap_or(0);

    let base_ap = game_data
        .battle
        .iter()
        .find(|t| t.id == battle_id)
        .map(|t| t.player_max)
        .unwrap_or(0);

    let hero_ap = match hero_count {
        0 | 1 | 2 => 2,
        _ => 4, // 3+
    };

    // Use the smaller of config vs hero-based cap
    base_ap.min(hero_ap)
}
