use super::BattleContext;
use super::entity_builder;
use anyhow::Result;

use database::models::game::heros::{HeroModel, UserHeroModel};

use once_cell::sync::Lazy;
use sonettobuf::{Fight, FightTeam};
use sqlx::SqlitePool;
use std::collections::HashMap;

pub async fn build_fight(
    pool: &SqlitePool,
    ctx: &BattleContext,
    fight_group: &sonettobuf::FightGroup,
) -> Result<Fight> {
    // Build attacker team (player)
    let attacker = build_attacker_team(pool, ctx.player_id, fight_group).await?;

    // Build defender team (enemies from episode config)
    let defender = build_defender_team(ctx.episode_id).await?;

    Ok(Fight {
        attacker: Some(attacker),
        defender: Some(defender.team),
        cur_round: Some(1),
        max_round: Some(defender.max_round),
        is_finish: Some(false), // determines if fight is over
        cur_wave: Some(1),
        battle_id: Some(ctx.battle_id),
        magic_circle: None,
        version: Some(5),
        is_record: Some(false), // enables sweep feature
        episode_id: Some(ctx.episode_id),
        fight_act_type: Some(sonettobuf::fight::FightActType::Normal.into()),
        last_change_hero_uid: Some(0),
        progress: Some(0),
        progress_max: Some(0),
        param: vec![],
        custom_data: vec![],
        fight_task_box: Some(sonettobuf::FightTaskBox { tasks: vec![] }),
        progress_list: vec![],
    })
}

static TRIAL_UID_MAP: Lazy<HashMap<i64, i32>> = Lazy::new(|| {
    let game_data = data::exceldb::get();
    let mut map = HashMap::new();

    let trial_heroes: Vec<_> = game_data.hero_trial.iter().collect();

    for (index, trial) in trial_heroes.iter().enumerate() {
        let uid = -((index + 1) as i64); // -1, -2, -3, ...
        map.insert(uid, trial.id);
        tracing::debug!("Trial mapping: UID {} -> trial_id {}", uid, trial.id);
    }

    map
});

async fn build_attacker_team(
    pool: &SqlitePool,
    user_id: i64,
    fight_group: &sonettobuf::FightGroup,
) -> Result<FightTeam> {
    let mut entitys = Vec::new();
    let mut sub_entitys = Vec::new();
    let hero = UserHeroModel::new(user_id, pool.clone());

    // Main heroes
    for (position, hero_uid) in fight_group.hero_list.iter().enumerate() {
        if *hero_uid == 0 {
            continue;
        }
        let hero_data = hero.get_uid(*hero_uid as i32).await?;
        let entity =
            entity_builder::build_hero_entity(pool, &hero_data, (position + 1) as i32, 1, false)
                .await;
        entitys.push(entity);
    }

    // Sub heroes
    for hero_uid in fight_group.sub_hero_list.iter() {
        if *hero_uid == 0 {
            continue;
        }
        let hero_data = hero.get_uid(*hero_uid as i32).await?;
        let entity = entity_builder::build_hero_entity(pool, &hero_data, -1, 1, true).await;
        sub_entitys.push(entity);
    }

    let player_entity = entity_builder::build_player_entity(user_id, 1);

    Ok(build_fight_team(
        entitys,
        sub_entitys,
        player_entity,
        Some(15),
        fight_group.cloth_id,
        build_player_skills(fight_group.cloth_id),
    ))
}

#[allow(dead_code)]
fn build_trial_hero_entity(
    hero_uid: i64,
    position: i32,
    team_type: i32,
) -> Result<sonettobuf::FightEntityInfo> {
    use data::exceldb;
    use sonettobuf::{EquipRecord, FightEntityInfo, HeroAttribute};

    let game_data = exceldb::get();

    // Look up trial hero using the map
    let trial_id = TRIAL_UID_MAP
        .get(&hero_uid)
        .ok_or_else(|| anyhow::anyhow!("Unknown trial hero UID: {}", hero_uid))?;

    let trial_data = game_data
        .hero_trial
        .get(*trial_id)
        .ok_or_else(|| anyhow::anyhow!("Trial data not found for ID {}", trial_id))?;

    tracing::info!(
        "Building trial hero entity: UID {} -> trial_id {} -> hero_id {}, level {}",
        hero_uid,
        trial_id,
        trial_data.hero_id,
        trial_data.level
    );

    // Get hero config for skills and career
    let hero_config = game_data
        .character
        .iter()
        .find(|h| h.id == trial_data.hero_id)
        .ok_or_else(|| {
            anyhow::anyhow!("Hero config not found for hero_id {}", trial_data.hero_id)
        })?;

    // Try to find exact level first
    let char_level_opt = game_data
        .character_level
        .iter()
        .find(|c| c.hero_id == trial_data.hero_id && c.level == trial_data.level);

    let (hp, attack, defense, mdefense, technic) = if let Some(char_level) = char_level_opt {
        // Found exact level
        (
            char_level.hp,
            char_level.atk,
            char_level.def,
            char_level.mdef,
            char_level.technic,
        )
    } else {
        // Level not found - try level 1 as base
        let base_level = game_data
            .character_level
            .iter()
            .find(|c| c.hero_id == trial_data.hero_id && c.level == 1)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No level data found for hero_id {} (tried level {} and level 1)",
                    trial_data.hero_id,
                    trial_data.level
                )
            })?;

        tracing::warn!(
            "Level {} not found for hero {}, using level 1 base stats",
            trial_data.level,
            trial_data.hero_id
        );

        // Use level 1 stats as base (you could calculate growth here if needed)
        (
            base_level.hp,
            base_level.atk,
            base_level.def,
            base_level.mdef,
            base_level.technic,
        )
    };

    // Parse skills from hero config
    let skill_group1 = parse_skill_group(&hero_config.skill, 1);
    let skill_group2 = parse_skill_group(&hero_config.skill, 2);

    // Passive skills - empty for trial heroes
    let passive_skill: Vec<i32> = vec![];

    // Get ex skill from hero config
    let ex_skill = hero_config.ex_skill;

    Ok(FightEntityInfo {
        uid: Some(hero_uid),
        model_id: Some(trial_data.hero_id),
        skin: Some(trial_data.skin),
        position: Some(position),
        entity_type: Some(1), // 1 = Hero
        user_id: Some(0),     // Trial heroes have no owner
        ex_point: Some(0),
        level: Some(trial_data.level),
        current_hp: Some(hp),
        attr: Some(HeroAttribute {
            hp: Some(hp),
            attack: Some(attack),
            defense: Some(defense),
            mdefense: Some(mdefense),
            technic: Some(technic),
            multi_hp_idx: Some(0),
            multi_hp_num: Some(0),
        }),
        buffs: vec![],
        skill_group1,
        skill_group2,
        passive_skill,
        ex_skill: Some(ex_skill),
        shield_value: Some(0),
        no_effect_buffs: vec![],
        expoint_max_add: Some(0),
        buff_harm_statistic: Some(0),
        equip_uid: Some(0),
        trial_equip: Some(EquipRecord {
            equip_uid: Some(0),
            equip_id: Some(trial_data.equip_id),
            equip_lv: Some(trial_data.equip_lv),
            refine_lv: Some(trial_data.equip_refine),
        }),
        ex_skill_level: Some(trial_data.ex_skill_lv),
        power_infos: vec![],
        act104_equip_uids: vec![],
        trial_act104_equips: vec![],
        summoned_list: vec![],
        base_attr: Some(HeroAttribute {
            hp: Some(hp),
            attack: Some(attack),
            defense: Some(defense),
            mdefense: Some(mdefense),
            technic: Some(technic),
            multi_hp_idx: Some(0),
            multi_hp_num: Some(0),
        }),
        ex_skill_point_change: Some(0),
        team_type: Some(team_type),
        enhance_info_box: Some(sonettobuf::EnhanceInfoBox {
            uid: Some(hero_uid),
            can_upgrade_ids: vec![],
            upgraded_options: vec![],
        }),
        trial_id: Some(trial_data.id),
        career: Some(hero_config.career),
        status: Some(0),
        guard: Some(-1),
        sub_cd: Some(0),
        ex_point_type: Some(0),
        equips: vec![],
        destiny_stone: Some(0),
        destiny_rank: Some(0),
        custom_unit_id: Some(0),
    })
}

// Helper function to parse skill groups
fn parse_skill_group(skill_str: &str, target_group: i32) -> Vec<i32> {
    // Parse: "1#31240111#31240112#31240113|2#31240121#31240122#31240123"
    for group_str in skill_str.split('|') {
        let parts: Vec<&str> = group_str.split('#').collect();
        if let Some(first) = parts.first()
            && let Ok(group_num) = first.parse::<i32>()
            && group_num == target_group
        {
            return parts[1..]
                .iter()
                .filter_map(|s| s.parse::<i32>().ok())
                .collect();
        }
    }
    vec![]
}

pub struct BattleSetup {
    pub max_round: i32,
    pub team: FightTeam,
}

async fn build_defender_team(episode_id: i32) -> Result<BattleSetup> {
    use data::exceldb;
    let game_data = exceldb::get();

    let episode = game_data
        .episode
        .iter()
        .find(|e| e.id == episode_id)
        .ok_or_else(|| anyhow::anyhow!("Episode {} not found", episode_id))?;

    let battle = game_data
        .battle
        .iter()
        .find(|b| b.id == episode.battle_id)
        .ok_or_else(|| anyhow::anyhow!("Battle {} not found", episode.battle_id))?;

    let max_round = battle.max_round;

    tracing::info!(
        "Loading battle {}: monsterGroupIds={}, maxRound={}",
        episode.battle_id,
        battle.monster_group_ids,
        max_round
    );

    let monster_ids: Vec<i32> = battle
        .monster_group_ids
        .split('#')
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();

    let mut entitys = Vec::new();
    for (idx, monster_id) in monster_ids.iter().enumerate() {
        let entity = build_enemy_entity(*monster_id, idx, (idx + 1) as i32, 2)?;

        tracing::info!(
            "Enemy entity: monster_id={}, position={}, uid={:?}",
            monster_id,
            idx + 1,
            entity.uid
        );

        entitys.push(entity);
    }

    tracing::info!("Built {} enemy entities", entitys.len());

    let player_entity = entity_builder::build_player_entity(0, 2);

    let fight_team = build_fight_team(entitys, vec![], player_entity, Some(0), Some(0), vec![]);

    Ok(BattleSetup {
        max_round,
        team: fight_team,
    })
}

fn build_fight_team(
    entitys: Vec<sonettobuf::FightEntityInfo>,
    sub_entitys: Vec<sonettobuf::FightEntityInfo>,
    player_entity: sonettobuf::FightEntityInfo,
    power: Option<i32>,
    cloth_id: Option<i32>,
    skill_infos: Vec<sonettobuf::PlayerSkillInfo>,
) -> FightTeam {
    FightTeam {
        entitys,
        sub_entitys,
        power,
        cloth_id,
        skill_infos,
        sp_entitys: vec![],
        indicators: vec![],
        ex_team_str: Some(String::new()),
        assist_boss: None,
        assist_boss_info: None,
        emitter: None,
        emitter_info: None,
        player_entity: Some(player_entity),
        player_finisher_info: None,
        energy: Some(0),
        card_heat: Some(sonettobuf::CardHeatInfo { values: vec![] }),
        card_deck_size: Some(0),
        blood_pool: None,
        vorpalith: None,
        item_skill_group: None,
        sp_fight_entities: vec![],
    }
}

fn build_enemy_entity(
    monster_id: i32,
    idx: usize,
    position: i32,
    team_type: i32,
) -> Result<sonettobuf::FightEntityInfo> {
    use data::exceldb;
    use sonettobuf::{EquipRecord, FightEntityInfo, HeroAttribute};

    let game_data = exceldb::get();

    let monster = game_data
        .monster
        .iter()
        .find(|m| m.id == monster_id)
        .ok_or_else(|| anyhow::anyhow!("Monster {} not found", monster_id))?;

    let template_id = if monster.template != 0 {
        monster.template
    } else {
        monster.skill_template
    };

    let template = game_data
        .monster_template
        .iter()
        .find(|t| t.template == template_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Monster template {} not found (monster {})",
                template_id,
                monster_id
            )
        })?;

    let skill_template = game_data
        .monster_skill_template
        .iter()
        .find(|s| s.id == monster.skill_template)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Monster skill template {} not found (monster {})",
                monster.skill_template,
                monster_id
            )
        })?;

    // Calculate stats based on level
    let level = if monster.level_true != 0 {
        monster.level_true
    } else {
        monster.level
    };

    let hp = template.life + (template.life_grow * level);
    let attack = template.attack + (template.attack_grow * level);
    let defense = template.defense + (template.defense_grow * level);
    let mdefense = template.mdefense + (template.mdefense_grow * level);
    let technic = template.technic + (template.technic_grow * level);

    // Parse skills: "1#40212511#40212512|2#40212521#40212522"
    let skill_group1 = parse_monster_skill_group(&skill_template.active_skill, 1);
    let skill_group2 = parse_monster_skill_group(&skill_template.active_skill, 2);

    // Parse passive skills: "2108" or "2108#2109"
    let passive_skill: Vec<i32> = skill_template
        .passive_skill
        .split('#')
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();

    // Get ex skill (first skill from uniqueSkill)
    let ex_skill = skill_template
        .unique_skill
        .split('#')
        .next()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);

    // Negative UIDs for enemies (-1, -2, -3, etc)
    let uid = -((idx + 1) as i64);

    tracing::debug!(
        "Enemy: monster_id={}, skill_template={}, uid={}, hp={}, skills1={:?}",
        monster_id,
        monster.skill_template,
        uid,
        hp,
        skill_group1
    );

    Ok(FightEntityInfo {
        uid: Some(uid),
        model_id: Some(monster.id), // Use monster.id as model_id
        skin: Some(monster.skin_id),
        position: Some(position),
        entity_type: Some(2), // 2 = Enemy
        user_id: Some(0),
        ex_point: Some(0),
        level: Some(level),
        current_hp: Some(hp),
        attr: Some(HeroAttribute {
            hp: Some(hp),
            attack: Some(attack),
            defense: Some(defense),
            mdefense: Some(mdefense),
            technic: Some(technic),
            multi_hp_idx: Some(0),
            multi_hp_num: Some(0),
        }),
        buffs: vec![],
        skill_group1,
        skill_group2,
        passive_skill,
        ex_skill: Some(ex_skill),
        shield_value: Some(0),
        no_effect_buffs: vec![],
        expoint_max_add: Some(0),
        buff_harm_statistic: Some(0),
        equip_uid: Some(0),
        trial_equip: Some(EquipRecord {
            equip_uid: Some(0),
            equip_id: Some(0),
            equip_lv: Some(0),
            refine_lv: Some(0),
        }),
        ex_skill_level: Some(0),
        power_infos: vec![],
        act104_equip_uids: vec![],
        trial_act104_equips: vec![],
        summoned_list: vec![],
        base_attr: Some(HeroAttribute {
            hp: Some(hp),
            attack: Some(attack),
            defense: Some(defense),
            mdefense: Some(mdefense),
            technic: Some(technic),
            multi_hp_idx: Some(0),
            multi_hp_num: Some(0),
        }),
        ex_skill_point_change: Some(0),
        team_type: Some(team_type),
        enhance_info_box: Some(sonettobuf::EnhanceInfoBox {
            uid: Some(uid),
            can_upgrade_ids: vec![],
            upgraded_options: vec![],
        }),
        trial_id: Some(0),
        career: Some(skill_template.career),
        status: Some(0),
        guard: Some(-1),
        sub_cd: Some(0),
        ex_point_type: Some(0),
        equips: vec![],
        destiny_stone: Some(0),
        destiny_rank: Some(0),
        custom_unit_id: Some(0),
    })
}

fn parse_monster_skill_group(active_skill: &str, target_group: i32) -> Vec<i32> {
    // Parse: "1#40212511#40212512|2#40212521#40212522"
    for group_str in active_skill.split('|') {
        let parts: Vec<&str> = group_str.split('#').collect();

        if let Some(first) = parts.first()
            && let Ok(group_num) = first.parse::<i32>()
            && group_num == target_group
        {
            return parts[1..]
                .iter()
                .filter_map(|s| s.parse::<i32>().ok())
                .collect();
        }
    }

    vec![]
}

fn build_player_skills(cloth_id: Option<i32>) -> Vec<sonettobuf::PlayerSkillInfo> {
    use data::exceldb;

    let game_data = exceldb::get();
    let cloth_id = cloth_id.unwrap_or(1);

    let cloth_level = game_data
        .cloth_level
        .iter()
        .find(|c| c.id == cloth_id && c.level == 1);

    if let Some(cloth) = cloth_level {
        let mut skills = Vec::new();

        // Skill 1
        if cloth.skill1 != 0 {
            skills.push(sonettobuf::PlayerSkillInfo {
                skill_id: Some(cloth.skill1),
                cd: Some(cloth.cd1),
                need_power: Some(cloth.use_power1.first().copied().unwrap_or(0)),
                r#type: Some(0),
            });
        }

        // Skill 2
        if cloth.skill2 != 0 {
            skills.push(sonettobuf::PlayerSkillInfo {
                skill_id: Some(cloth.skill2),
                cd: Some(cloth.cd2),
                need_power: Some(cloth.use_power2.first().copied().unwrap_or(0)),
                r#type: Some(0),
            });
        }

        // Skill 3
        if cloth.skill3 != 0 {
            skills.push(sonettobuf::PlayerSkillInfo {
                skill_id: Some(cloth.skill3),
                cd: Some(cloth.cd3),
                need_power: None,
                r#type: Some(0),
            });
        }

        return skills;
    }

    vec![]
}
