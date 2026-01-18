use data::exceldb;
use database::{
    db::game::equipment::Equipment,
    models::game::{equipment::UserEquipmentModel, heros::HeroData},
};
use sonettobuf::{EquipRecord, FightEntityInfo, HeroAttribute};
use sqlx::SqlitePool;
use std::collections::HashMap;

pub async fn build_hero_entity(
    pool: &SqlitePool,
    hero_data: &HeroData,
    position: i32,
    team_type: i32,
    is_sub: bool,
) -> FightEntityInfo {
    let record = &hero_data.record;

    let equip_model = UserEquipmentModel::new(record.user_id, pool.clone());
    let equip_data = equip_model.get_equip(record.default_equip_uid).await.ok();

    let equip_id = equip_data.as_ref().map(|equip| equip.equip_id);

    let game = exceldb::get();
    let hero_type = game
        .character
        .iter()
        .find(|c| c.id == record.hero_id)
        .map(|c| c.hero_type)
        .unwrap_or(1);

    let ex_level = record.ex_skill_level;

    let destiny_active = record.destiny_stone > 0 && record.destiny_rank > 0;

    let destiny_exchange = if destiny_active {
        Some(get_destiny_exchange_map(
            record.destiny_stone,
            record.destiny_rank,
        ))
    } else {
        None
    };

    let (mut skill_group1, mut skill_group2) =
        if let Some((sg1, sg2, _)) = lookup_activity174_kit(record.hero_id) {
            (sg1, sg2)
        } else if is_sub {
            (
                get_skill_from_character(record.hero_id, 1),
                get_skill_from_character(record.hero_id, 2),
            )
        } else {
            (
                resolve_skill_group(record.hero_id, 1, ex_level, hero_type),
                resolve_skill_group(record.hero_id, 2, ex_level, hero_type),
            )
        };

    let passives = resolve_passives(hero_data, equip_id, destiny_exchange.as_ref());

    if let Some(map) = &destiny_exchange {
        apply_destiny_exchange(&mut skill_group1, map);
        apply_destiny_exchange(&mut skill_group2, map);
    }

    let attr = build_attr(hero_data, equip_data.as_ref());
    let current_hp = attr.hp.unwrap_or(0);

    let initial_ex_point = calculate_initial_ex_point(record.hero_id, &passives);

    FightEntityInfo {
        uid: Some(record.uid),
        model_id: Some(record.hero_id),
        skin: Some(record.skin),
        position: Some(position),
        entity_type: Some(1),
        user_id: Some(record.user_id),
        ex_point: Some(initial_ex_point),
        level: Some(record.level),
        current_hp: Some(current_hp),
        attr: Some(attr),
        buffs: vec![],
        skill_group1,
        skill_group2,
        passive_skill: passives,
        ex_skill: Some(get_hero_ex_skill(hero_data, destiny_exchange.as_ref())),

        shield_value: Some(0),
        no_effect_buffs: vec![],
        expoint_max_add: Some(0),
        buff_harm_statistic: Some(0),
        equip_uid: Some(record.default_equip_uid),
        trial_equip: Some(EquipRecord {
            equip_uid: Some(0),
            equip_id: Some(0),
            equip_lv: Some(0),
            refine_lv: Some(0),
        }),
        ex_skill_level: Some(record.ex_skill_level),
        power_infos: vec![],
        act104_equip_uids: vec![],
        trial_act104_equips: vec![],
        summoned_list: vec![],
        base_attr: Some(attr),
        ex_skill_point_change: Some(0),
        team_type: Some(team_type),
        enhance_info_box: Some(sonettobuf::EnhanceInfoBox {
            uid: Some(record.uid),
            can_upgrade_ids: vec![],
            upgraded_options: vec![],
        }),
        trial_id: Some(0),
        career: Some(get_hero_career(hero_data)),
        status: Some(0),
        guard: Some(-1),
        sub_cd: Some(0),
        ex_point_type: Some(detect_ex_point_type(record.hero_id)),
        equips: vec![sonettobuf::EquipRecord {
            equip_uid: equip_data.as_ref().map(|e| e.uid),
            equip_id: equip_data.as_ref().map(|e| e.equip_id),
            equip_lv: equip_data.as_ref().map(|e| e.level),
            refine_lv: equip_data.as_ref().map(|e| e.refine_lv),
        }],
        destiny_stone: Some(record.destiny_stone),
        destiny_rank: Some(record.destiny_rank),
        custom_unit_id: Some(0),
    }
}

fn build_attr(r: &HeroData, equip: Option<&Equipment>) -> HeroAttribute {
    let mut hp = ((r.record.base_hp as f32) * 1.0986541).round() as i32;
    let mut atk = ((r.record.base_attack as f32) * 1.0786).round() as i32;
    let mut def = ((r.record.base_defense as f32) * 1.0942857).round() as i32;
    let mut mdef = ((r.record.base_mdefense as f32) * 1.0942857).round() as i32;
    let technic = ((r.record.base_technic as f32) * 1.395604).round() as i32;

    if let Some(equip) = equip {
        let game_data = exceldb::get();

        if let Some(strengthen) = game_data
            .equip_strengthen
            .iter()
            .find(|s| s.strength_type == equip.equip_id)
        {
            hp += strengthen.hp;
            atk += strengthen.atk;
            def += strengthen.def;
            mdef += strengthen.mdef;
        }
    }

    HeroAttribute {
        hp: Some(hp),
        attack: Some(atk),
        defense: Some(def),
        mdefense: Some(mdef),
        technic: Some(technic),
        multi_hp_idx: Some(r.record.base_multi_hp_idx),
        multi_hp_num: Some(r.record.base_multi_hp_num),
    }
}

pub fn build_player_entity(user_id: i64, team_type: i32) -> FightEntityInfo {
    let uid = if team_type == 1 { 0 } else { -99999 };

    FightEntityInfo {
        uid: Some(uid),
        model_id: Some(0),
        skin: Some(0),
        position: Some(0),
        entity_type: Some(3),
        user_id: Some(user_id),
        ex_point: Some(0),
        level: Some(0),
        current_hp: Some(100),
        attr: Some(HeroAttribute {
            hp: Some(100),
            attack: Some(0),
            defense: Some(0),
            mdefense: Some(0),
            technic: Some(0),
            multi_hp_idx: Some(0),
            multi_hp_num: Some(0),
        }),
        buffs: vec![],
        skill_group1: vec![],
        skill_group2: vec![],
        passive_skill: vec![],
        ex_skill: Some(0),
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
        base_attr: None,
        ex_skill_point_change: Some(0),
        team_type: Some(team_type),
        enhance_info_box: Some(sonettobuf::EnhanceInfoBox {
            uid: Some(uid),
            can_upgrade_ids: vec![],
            upgraded_options: vec![],
        }),
        trial_id: Some(0),
        career: Some(0),
        status: Some(0),
        guard: Some(-1),
        sub_cd: Some(0),
        ex_point_type: Some(0),
        equips: vec![],
        destiny_stone: Some(0),
        destiny_rank: Some(0),
        custom_unit_id: Some(0),
    }
}

pub fn resolve_skill_group(hero_id: i32, group: i32, ex_level: i32, hero_type: i32) -> Vec<i32> {
    let ex = lookup_ex_skill_group(hero_id, group, ex_level);

    if !ex.is_empty() {
        parse_ex_skill_string(ex, hero_type)
    } else {
        lookup_base_skill_group(hero_id, group)
    }
}

fn parse_ex_skill_string(s: &str, hero_type: i32) -> Vec<i32> {
    if s.contains(',') {
        let sets: Vec<&str> = s.split(',').collect();
        let idx = if hero_type < 1 || hero_type as usize > sets.len() {
            0
        } else {
            (hero_type - 1) as usize
        };

        sets.get(idx)
            .into_iter()
            .flat_map(|v| v.split('|'))
            .filter_map(|v| v.parse().ok())
            .collect()
    } else {
        s.split('|').filter_map(|v| v.parse().ok()).collect()
    }
}

fn lookup_base_skill_group(hero_id: i32, group: i32) -> Vec<i32> {
    let game = exceldb::get();

    let Some(c) = game.character.iter().find(|c| c.id == hero_id) else {
        return vec![];
    };

    for block in c.skill.split('|') {
        let mut parts = block.split('#');
        let Ok(g) = parts.next().unwrap_or("").parse::<i32>() else {
            continue;
        };

        if g == group {
            return parts.filter_map(|v| v.parse().ok()).collect();
        }
    }

    vec![]
}

fn calculate_initial_ex_point(hero_id: i32, _passives: &[i32]) -> i32 {
    match hero_id {
        3088 => 0,
        3120 => 0,
        _ => 0,
    }
}

fn detect_ex_point_type(hero_id: i32) -> i32 {
    match hero_id {
        3120 => 1,
        3123 => 2,
        3124 | 3122 => 3,
        _ => 0,
    }
}

fn lookup_ex_skill_group(hero_id: i32, group: i32, ex_level: i32) -> &'static str {
    let game = exceldb::get();

    for lvl in (1..=ex_level).rev() {
        if let Some(ex) = game
            .skill_ex_level
            .iter()
            .find(|s| s.hero_id == hero_id && s.skill_level == lvl)
        {
            match group {
                1 if !ex.skill_group1.is_empty() => return &ex.skill_group1,
                2 if !ex.skill_group2.is_empty() => return &ex.skill_group2,
                _ => {}
            }
        }
    }

    ""
}

fn get_skill_from_character(hero_id: i32, group: i32) -> Vec<i32> {
    let game_data = exceldb::get();

    let Some(character) = game_data.character.iter().find(|c| c.id == hero_id) else {
        tracing::warn!("Character {} not found in character table", hero_id);
        return vec![];
    };

    if character.skill.is_empty() {
        return vec![];
    }

    // "1#31250111#31250112#31250113|2#31250121#31250122#31250123"
    for block in character.skill.split('|') {
        let mut parts = block.split('#');

        let Some(group_str) = parts.next() else {
            continue;
        };
        let Ok(g) = group_str.parse::<i32>() else {
            continue;
        };

        if g != group {
            continue;
        }

        let skills: Vec<i32> = parts.filter_map(|s| s.parse::<i32>().ok()).collect();

        tracing::info!(
            "Using character.skill fallback for hero {} group {}: {:?}",
            hero_id,
            group,
            skills
        );

        return skills;
    }

    vec![]
}

fn get_destiny_exchange_map(facets_id: i32, level: i32) -> HashMap<i32, i32> {
    let game = exceldb::get();

    game.character_destiny_facets
        .iter()
        .find(|f| f.facets_id == facets_id && f.level == level)
        .map(|f| parse_exchange_skills(&f.exchange_skills))
        .unwrap_or_default()
}

fn parse_exchange_skills(s: &str) -> HashMap<i32, i32> {
    let mut map = HashMap::new();

    for pair in s.split('|') {
        if let Some((old, new)) = pair.split_once('#')
            && let (Ok(o), Ok(n)) = (old.parse(), new.parse())
        {
            map.insert(o, n);
        }
    }

    map
}

fn apply_destiny_exchange(list: &mut [i32], map: &HashMap<i32, i32>) {
    for v in list.iter_mut() {
        if let Some(new) = map.get(v) {
            *v = *new;
        }
    }
}

fn lookup_activity174_kit(hero_id: i32) -> Option<(Vec<i32>, Vec<i32>, Vec<i32>)> {
    let game = exceldb::get();

    let role = game
        .activity174_role
        .iter()
        .find(|r| r.hero_id == hero_id)?;

    let sg1 = role
        .active_skill1
        .split('#')
        .filter_map(|v| v.parse().ok())
        .collect();

    let sg2 = role
        .active_skill2
        .split('#')
        .filter_map(|v| v.parse().ok())
        .collect();

    let passives = role
        .passive_skill
        .split('|')
        .filter_map(|v| v.parse().ok())
        .collect();

    Some((sg1, sg2, passives))
}

fn resolve_passives(
    hero_data: &HeroData,
    equip_id: Option<i32>,
    destiny_exchange: Option<&HashMap<i32, i32>>,
) -> Vec<i32> {
    let game = exceldb::get();
    let hero_id = hero_data.record.hero_id;
    let ex_level = hero_data.record.ex_skill_level;
    let destiny_active = destiny_exchange.is_some();

    let mut passives: Vec<i32> = Vec::new();

    let activity_base: Option<Vec<i32>> =
        if let Some(r) = game.activity174_role.iter().find(|r| r.hero_id == hero_id) {
            if !r.passive_skill.is_empty() {
                Some(
                    r.passive_skill
                        .split('|')
                        .filter_map(|v| v.parse::<i32>().ok())
                        .collect(),
                )
            } else {
                None
            }
        } else if let Some(r) = game.activity191_role.iter().find(|r| r.role_id == hero_id) {
            if !r.passive_skill.is_empty() {
                Some(
                    r.passive_skill
                        .split('|')
                        .filter_map(|v| v.parse::<i32>().ok())
                        .collect(),
                )
            } else {
                None
            }
        } else {
            None
        };

    if let Some(base_display_ids) = activity_base {
        let mut map = HashMap::<i32, i32>::new();

        for lvl in 1..=ex_level {
            if let Some(ex) = game
                .skill_ex_level
                .iter()
                .find(|s| s.hero_id == hero_id && s.skill_level == lvl)
            {
                for pair in ex.passive_skill.split('|') {
                    if let Some((display, actual)) = pair.split_once('#')
                        && let (Ok(d), Ok(a)) = (display.parse::<i32>(), actual.parse::<i32>())
                    {
                        map.insert(d, a);
                    }
                }
            }
        }

        for d in base_display_ids {
            let ex_resolved = *map.get(&d).unwrap_or(&d);

            let final_id = if let Some(destiny) = destiny_exchange {
                *destiny.get(&d).unwrap_or(&ex_resolved)
            } else {
                ex_resolved
            };

            passives.push(final_id);
        }
    } else {
        let mut rows: Vec<(i32, i32)> = game
            .skill_passive_level
            .iter()
            .filter(|s| s.hero_id == hero_id && s.skill_passive != 0)
            .map(|s| (s.skill_level, s.skill_passive))
            .collect();

        rows.sort_by_key(|(level, _)| if *level == 0 { i32::MAX } else { *level });

        for (_, id) in rows {
            passives.push(id);
        }
    }

    if hero_id == 3088 && destiny_active {
        let bonus = [308801911, 308801921, 308802111];
        for id in bonus {
            if !passives.contains(&id) {
                passives.push(id);
            }
        }
    }

    if hero_id == 3126 && !passives.contains(&31260191) {
        passives.push(31260191);
    }

    if let Some(eid) = equip_id
        && let Some(e) = game.equip_skill.iter().find(|e| e.id == eid)
    {
        if e.skill != 0 {
            passives.push(e.skill);
        }
        if e.skill2 != 0 {
            passives.push(e.skill2);
        }
    }

    passives
}

fn get_hero_ex_skill(hero_data: &HeroData, destiny_exchange: Option<&HashMap<i32, i32>>) -> i32 {
    let game = exceldb::get();
    let hero_id = hero_data.record.hero_id;
    let ex_level = hero_data.record.ex_skill_level;

    let mut ex_skill = game
        .skill_ex_level
        .iter()
        .find(|s| s.hero_id == hero_id && s.skill_level == ex_level)
        .map(|s| s.skill_ex)
        .unwrap_or(0);

    if ex_skill == 0 {
        ex_skill = game
            .skill_ex_level
            .iter()
            .find(|s| s.hero_id == hero_id && s.skill_level == 1)
            .map(|s| s.skill_ex)
            .unwrap_or(0);
    }

    if let Some(map) = destiny_exchange
        && let Some(replaced) = map.get(&ex_skill)
    {
        ex_skill = *replaced;
    }

    ex_skill
}

fn get_hero_career(hero_data: &HeroData) -> i32 {
    let game_data = exceldb::get();
    let hero_id = hero_data.record.hero_id;

    game_data
        .character
        .iter()
        .find(|s| s.id == hero_id)
        .map(|s| s.career)
        .unwrap_or(0)
}
