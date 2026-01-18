use crate::error::AppError;
use data::exceldb;
use database::models::game::heros::{HeroModel, UserHeroModel};
use once_cell::sync::Lazy;
use rand::thread_rng;
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};
use sonettobuf::{CardInfo, CardInfoPush, Fight, FightGroup};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

static CARD_UID: AtomicI64 = AtomicI64::new(1);

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

pub async fn generate_ai_initial_deck(fight: &Fight, seed: u64) -> Vec<CardInfo> {
    let mut rng = StdRng::seed_from_u64(seed);

    let mut cards = Vec::new();

    let Some(attacker) = &fight.attacker else {
        return cards;
    };
    let Some(defender) = &fight.defender else {
        return cards;
    };

    let players: Vec<i64> = attacker
        .entitys
        .iter()
        .filter_map(|e| {
            let uid = e.uid.unwrap_or(0);
            let hp = e.current_hp.unwrap_or(0);
            if uid > 0 && hp > 0 { Some(uid) } else { None }
        })
        .collect();

    if players.is_empty() {
        return cards;
    }

    for enemy in &defender.entitys {
        let enemy_uid = enemy.uid.unwrap_or(0);
        if enemy_uid >= 0 {
            continue;
        }
        if enemy.current_hp.unwrap_or(0) <= 0 {
            continue;
        }

        let skill_id = enemy.skill_group1.first().copied().unwrap_or(0);
        if skill_id == 0 {
            continue;
        }

        let target_uid = players[rng.gen_range(0..players.len())];

        let game_data = data::exceldb::get();
        let skill_effect_id = game_data
            .skill
            .iter()
            .find(|s| s.id == skill_id)
            .map(|s| s.skill_effect)
            .unwrap_or(0);

        if skill_effect_id == 0 {
            tracing::warn!("AI: skill {} has no skillEffect, skipping", skill_id);
            continue;
        }

        cards.push(CardInfo {
            uid: Some(enemy_uid),
            skill_id: Some(skill_id),
            card_effect: Some(0),
            temp_card: Some(false),
            enchants: vec![],
            card_type: Some(0),
            hero_id: enemy.model_id,
            status: Some(0),
            target_uid: Some(target_uid),
            extra_info: None,
            energy: Some(0),
            extra_infos: vec![],
            area_red_or_blue: Some(0),
            heat_id: Some(0),
        });
    }

    cards
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

static TRIAL_UID_MAP: Lazy<HashMap<i64, i32>> = Lazy::new(|| {
    let game_data = exceldb::get();
    let mut map = HashMap::new();

    let trial_heroes: Vec<_> = game_data.hero_trial.iter().collect();

    for (index, trial) in trial_heroes.iter().enumerate() {
        let uid = -((index + 1) as i64); // -1, -2, -3, ...
        map.insert(uid, trial.id);
        tracing::debug!("Trial mapping: UID {} -> trial_id {}", uid, trial.id);
    }

    map
});

async fn build_candidate_pool(
    pool: &SqlitePool,
    user_id: i64,
    hero_uids: &[i64],
) -> Result<Vec<CardInfo>, AppError> {
    let mut pool_cards = Vec::new();
    let game_data = exceldb::get();

    let hero = UserHeroModel::new(user_id, pool.clone());

    for &hero_uid in hero_uids {
        if hero_uid == 0 {
            continue; // Skip empty slots
        }

        let hero_id = if hero_uid < 0 {
            // === TRIAL HERO PATH - NO DATABASE ACCESS ===
            let trial_id = TRIAL_UID_MAP.get(&hero_uid).ok_or_else(|| {
                tracing::error!("Unknown trial hero UID: {}", hero_uid);
                AppError::InvalidRequest
            })?;

            let trial_data = game_data
                .hero_trial
                .iter()
                .find(|t| t.id == *trial_id)
                .ok_or_else(|| {
                    tracing::error!("Trial data not found for ID {}", trial_id);
                    AppError::InvalidRequest
                })?;

            tracing::info!(
                "Trial hero: UID {} -> trial_id {} -> hero_id {}",
                hero_uid,
                trial_id,
                trial_data.hero_id
            );

            trial_data.hero_id
        } else {
            // Regular hero - load from database
            let hero = hero.get_uid(hero_uid as i32).await?;
            hero.record.hero_id
        };

        // Get skills from static data (works for both trial and regular heroes)
        let skills = get_hero_skills(hero_id);

        for skill_id in skills {
            pool_cards.push(CardInfo {
                uid: Some(CARD_UID.fetch_add(1, Ordering::SeqCst)),
                hero_id: Some(hero_id),
                skill_id: Some(skill_id),
                card_type: Some(0),
                status: Some(0),
                temp_card: Some(hero_uid < 0), // Mark trial hero cards
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
        if parts.len() > 1
            && let Ok(skill_id) = parts[1].parse::<i32>()
        {
            skills.push(skill_id);
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
        0..=2 => 2,
        _ => 4, // 3+
    };

    // Use the smaller of config vs hero-based cap
    base_ap.min(hero_ap)
}
