use data::exceldb;
use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

/// Load CritterInfo starter data from critter_info.json
pub async fn load_critter_info(tx: &mut Transaction<'_, Sqlite>, uid: i64) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/critter_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_critter_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let array = match data.get("critterInfos").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => {
            eprintln!("load_critter_info: invalid or missing 'critterInfos' array");
            return Ok(());
        }
    };

    let now = common::time::ServerTime::now_ms();

    // Starting seed for critter UIDs - ignore the uid field in JSON
    let mut critter_uid = 10000000i64;

    for entry in array {
        // Note: We ignore entry.get("uid") and use our own counter
        let define_id = entry
            .get("defineId")
            .and_then(|v| v.as_i64())
            .unwrap_or_default();

        // Insert main critter record with generated UID
        sqlx::query(
            r#"
            INSERT INTO critters (
                uid, player_id, define_id, create_time,
                efficiency, patience, lucky,
                efficiency_incr_rate, patience_incr_rate, lucky_incr_rate,
                special_skin, current_mood, is_locked, finish_train, is_high_quality,
                train_hero_id, total_finish_count, name,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(critter_uid) // Generated UID, not from JSON
        .bind(uid)
        .bind(define_id)
        .bind(now) // Fresh timestamp
        .bind(entry.get("efficiency").and_then(|v| v.as_i64()))
        .bind(entry.get("patience").and_then(|v| v.as_i64()))
        .bind(entry.get("lucky").and_then(|v| v.as_i64()))
        .bind(entry.get("efficiencyIncrRate").and_then(|v| v.as_i64()))
        .bind(entry.get("patienceIncrRate").and_then(|v| v.as_i64()))
        .bind(entry.get("luckyIncrRate").and_then(|v| v.as_i64()))
        .bind(
            entry
                .get("specialSkin")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        )
        .bind(entry.get("currentMood").and_then(|v| v.as_i64()))
        .bind(entry.get("lock").and_then(|v| v.as_bool()).unwrap_or(false))
        .bind(
            entry
                .get("finishTrain")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        )
        .bind(
            entry
                .get("isHighQuality")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        )
        .bind(entry.get("trainHeroId").and_then(|v| v.as_i64()))
        .bind(entry.get("totalFinishCount").and_then(|v| v.as_i64()))
        .bind(entry.get("name").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        // Insert skill tags
        if let Some(skill_info) = entry.get("skillInfo").and_then(|v| v.as_object()) {
            if let Some(tags) = skill_info.get("tags").and_then(|v| v.as_array()) {
                for (i, tag) in tags.iter().enumerate() {
                    if let Some(tag_str) = tag.as_str() {
                        sqlx::query(
                            "INSERT INTO critter_skills (critter_uid, tag, sort_order) VALUES (?, ?, ?)"
                        )
                        .bind(critter_uid)
                        .bind(tag_str)
                        .bind(i as i32)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }
        }

        // Insert tag attribute rates
        if let Some(attr_rates) = entry.get("tagAttributeRates").and_then(|v| v.as_array()) {
            for attr in attr_rates {
                sqlx::query(
                    "INSERT INTO critter_tag_attributes (critter_uid, attribute_id, rate) VALUES (?, ?, ?)"
                )
                .bind(critter_uid)
                .bind(attr.get("attributeId").and_then(|v| v.as_i64()))
                .bind(attr.get("rate").and_then(|v| v.as_i64()))
                .execute(&mut **tx)
                .await?;
            }
        }

        // Insert rest info if present
        if let Some(rest_info) = entry.get("restInfo").and_then(|v| v.as_object()) {
            sqlx::query(
                "INSERT INTO critter_rest_info (critter_uid, building_uid, rest_slot_id) VALUES (?, ?, ?)"
            )
            .bind(critter_uid)
            .bind(rest_info.get("buildingUid").and_then(|v| v.as_i64()))
            .bind(rest_info.get("restSlotId").and_then(|v| v.as_i64()))
            .execute(&mut **tx)
            .await?;
        }

        // Increment UID for next critter
        critter_uid += 1;
    }

    tracing::info!(
        "Loaded {} critters for uid {} starting from UID 10000000",
        array.len(),
        uid
    );
    Ok(())
}

/// Load default player info for new user
pub async fn load_player_info(tx: &mut Transaction<'_, Sqlite>, uid: i64) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();

    // Create player_info record
    sqlx::query(
        "INSERT INTO player_info (
            player_id, signature, birthday, portrait, show_achievement, bg,
            total_login_days, last_episode_id, last_logout_time,
            hero_rare_nn_count, hero_rare_n_count, hero_rare_r_count,
            hero_rare_sr_count, hero_rare_ssr_count,
            created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
    )
    .bind(uid)
    .bind("Not a whale") // Default signature
    .bind("")
    .bind(171603)
    .bind("") // show achievement
    .bind(0) // bg
    .bind(0) // total_login_days
    .bind(0)
    .bind(None::<i64>)
    .bind(0) //hero_rare_nn_count
    .bind(0) //hero_rare_n_count
    .bind(0) //hero_rare_r_count
    .bind(0) // hero_rare_sr_count
    .bind(0) // hero_rare_ssr_count
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Add default show heroes (3 heroes for profile display)
    let default_heroes = [
        (3086, 180, 4, 5, 308603),
        (3120, 180, 4, 5, 312002),
        (3095, 180, 4, 5, 309502),
    ];

    for (i, (hero_id, level, rank, ex_skill_level, skin)) in default_heroes.iter().enumerate() {
        sqlx::query(
            "INSERT INTO player_show_heroes
             (player_id, hero_id, level, rank, ex_skill_level, skin, display_order)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uid)
        .bind(hero_id)
        .bind(level)
        .bind(rank)
        .bind(ex_skill_level)
        .bind(skin)
        .bind(i as i32)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

/// Generate and insert starter heroes from ExcelDB
pub async fn load_hero_list(
    tx: &mut Transaction<'_, Sqlite>,
    uid: i64,
    equip_map: &HashMap<i32, i64>,
) -> sqlx::Result<()> {
    let game_data = exceldb::get();
    let now = common::time::ServerTime::now_ms();

    // Starting seed for hero UIDs
    static HERO_UID_COUNTER: AtomicI64 = AtomicI64::new(20000000);

    let mut rarity_counts = vec![0i32; 6];

    // Get all characters (filter out NPCs)
    let characters: Vec<_> = game_data
        .character
        .iter()
        //.filter(|c| c.rare >= 1 && c.rare <= 5) // Valid rarity (1-5 stars)
        .filter(|c| c.id != 3029)
        .filter(|c| c.id != 9998)
        .collect();

    for character in characters.clone() {
        let hero_uid = HERO_UID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let hero_id = character.id;
        let hero_skin = character.skin_id;
        let rare = character.rare as usize;

        if (1..=5).contains(&rare) {
            rarity_counts[rare] += 1;
        }

        // Get MAX LEVEL stats (highest level available for this hero)
        let max_stats = game_data
            .character_level
            .iter()
            .filter(|s| s.hero_id == hero_id)
            .max_by_key(|s| s.level); // Get the highest level entry

        let (level, hp, atk, def, mdef, technic, cri, recri, cri_dmg, cri_def, add_dmg, drop_dmg) =
            if let Some(stats) = max_stats {
                (
                    stats.level,
                    stats.hp,
                    stats.atk,
                    stats.def,
                    stats.mdef,
                    stats.technic,
                    stats.cri,
                    stats.recri,
                    stats.cri_dmg,
                    stats.cri_def,
                    stats.add_dmg,
                    stats.drop_dmg,
                )
            } else {
                // Fallback values if no stats found
                (1, 1000, 100, 100, 100, 100, 0, 0, 1300, 0, 0, 0)
            };

        let max_ranks = game_data
            .character_rank
            .iter()
            .filter(|s| s.hero_id == hero_id)
            .max_by_key(|s| s.rank);

        // Determine max rank
        let max_rank = if let Some(max) = max_ranks {
            max.rank
        } else {
            3
        };

        // Get default skin - default is always heroId * 100 + 01
        let default_skin = game_data
            .skin
            .iter()
            .filter(|s| s.character_id != 0)
            .filter(|s| s.character_id == hero_id)
            .max_by_key(|s| s.id) // Latest skin ID will be the default
            .map(|s| s.id)
            .unwrap_or(hero_skin); // Fallback to i1 skin

        // Get default destiny stone from character_destiny table
        let destiny_data = game_data
            .character_destiny
            .iter()
            .find(|d| d.hero_id == hero_id);

        // Calculate destiny values ONLY if hero has destiny system
        let (destiny_rank, destiny_level, destiny_stone, red_dot_type) =
            if let Some(d) = destiny_data {
                // Hero has destiny - set to max
                let rank = max_rank; // Match hero's max rank
                let level = 10; // Max destiny level
                let stone = d
                    .facets_id
                    .split('#')
                    .next()
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(0);
                let red_dot_type = 6;
                (rank, level, stone, red_dot_type)
            } else {
                // Hero doesn't have destiny system - all zeros
                (0, 0, 0, 0)
            };

        // equipRec can be "1527" or "1530#1428" (multiple options); take first
        let equip_id = character
            .equip_rec
            .split('#')
            .next()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1501);

        // Get default equipment UID from the map using equip_id
        let default_equip_uid = equip_map.get(&equip_id).copied().unwrap_or(0);

        let strengthen_stats = game_data
            .equip_strengthen
            .iter()
            .find(|s| s.strength_type == equip_id);

        let (
            final_hp,
            final_atk,
            final_def,
            final_mdef,
            final_technic,
            final_cri,
            final_recri,
            final_cri_dmg,
            final_cri_def,
            final_add_dmg,
            final_drop_dmg,
        ) = if let Some(s) = strengthen_stats {
            (
                hp + s.hp,
                atk + s.atk,
                def + s.def,
                mdef + s.mdef,
                technic,
                cri,
                recri,
                cri_dmg,
                cri_def,
                add_dmg,
                drop_dmg,
            )
        } else {
            (
                hp, atk, def, mdef, technic, cri, recri, cri_dmg, cri_def, add_dmg, drop_dmg,
            )
        };

        let extra_str = if let 3123 = hero_id {
            "1003#2003"
        } else if hero_id == 3124 {
            "2#21,22|3#32,33,31"
        } else {
            ""
        };

        let max_talent_id = game_data
            .character_talent
            .iter()
            .filter(|t| t.hero_id == hero_id)
            .map(|t| t.talent_id)
            .max()
            .expect("Hero has no talent data");

        // Insert main hero record at MAX LEVEL with MAX RANK
        sqlx::query(
            r#"
            INSERT INTO heroes (
                uid, user_id, hero_id, create_time,
                level, exp, rank, breakthrough, skin, faith,
                active_skill_level, ex_skill_level, is_new, talent,
                default_equip_uid, duplicate_count, use_talent_template_id,
                talent_style_unlock, talent_style_red, is_favor,
                destiny_rank, destiny_level, destiny_stone, red_dot, extra_str,
                base_hp, base_attack, base_defense, base_mdefense, base_technic,
                base_multi_hp_idx, base_multi_hp_num,
                ex_cri, ex_recri, ex_cri_dmg, ex_cri_def, ex_add_dmg, ex_drop_dmg
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
                ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30,
                ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38
            )
            "#,
        )
        .bind(hero_uid)
        .bind(uid)
        .bind(hero_id)
        .bind(now)
        .bind(level) // MAX level (e.g., 180)
        .bind(0) // exp (maxed out, no exp needed)
        .bind(max_rank) // MAX rank based on rarity
        .bind(0) // breakthrough
        .bind(default_skin)
        .bind(10400) // faith (max)
        .bind(1) // Active skill level (max)
        .bind(5) // Ex skill level (max)
        .bind(false) // is_new
        .bind(max_talent_id) // talent
        .bind(default_equip_uid) // default_equip_uid
        .bind(5) // duplicate_count
        .bind(1) // use_talent_template_id
        .bind(1) // talent_style_unlock
        .bind(0) // talent_style_red
        .bind(false) // is_favor
        .bind(destiny_rank) // destiny_rank
        .bind(destiny_level) // destiny_level
        .bind(destiny_stone) // destiny_stone (from character_destiny table)
        .bind(red_dot_type) // red_dot
        .bind(extra_str) // extra_str
        // Base attributes (MAX level stats + equip strengthen bonuses)
        .bind(final_hp)
        .bind(final_atk)
        .bind(final_def)
        .bind(final_mdef)
        .bind(final_technic)
        .bind(0) // base_multi_hp_idx
        .bind(0) // base_multi_hp_num
        // Ex attributes
        .bind(final_cri)
        .bind(final_recri)
        .bind(final_cri_dmg)
        .bind(final_cri_def)
        .bind(final_add_dmg)
        .bind(final_drop_dmg)
        .execute(&mut **tx)
        .await?;

        // Insert passive skill levels in order
        // passiveSkillLevel is an array of levels: [1, 2, 3] means skill 0 is level 1, skill 1 is level 2, etc.
        let max_skill_group = game_data
            .skill_passive_level
            .iter()
            .filter(|s| s.hero_id == hero_id)
            .map(|s| s.skill_group)
            .max()
            .unwrap_or(0);

        // For each skill group (1, 2, 3...), get the max level
        for skill_group in 1..=max_skill_group {
            let max_level = game_data
                .skill_passive_level
                .iter()
                .filter(|s| s.hero_id == hero_id && s.skill_group == skill_group)
                .map(|s| s.skill_level)
                .max()
                .unwrap_or(0);

            // skill_index is 0-based, so skill_group 1 -> index 0
            sqlx::query(
                "INSERT INTO hero_passive_skill_levels (hero_uid, skill_index, level) VALUES (?, ?, ?)"
            )
            .bind(hero_uid)
            .bind(skill_group - 1) // Convert 1-based to 0-based
            .bind(max_level)
            .execute(&mut **tx)
            .await?;
        }

        // No passive skills at level 1 (empty array in example)
        // They unlock through leveling/breakthrough

        // Get all voices for this character from character_voice table
        let character_voices: Vec<&data::exceldb::character_voice::CharacterVoice> = game_data
            .character_voice
            .iter()
            .filter(|v| v.hero_id == hero_id)
            .filter(|t| t.r#type == 9 || t.r#type == 11)
            .collect();

        // Insert unlocked voices (voices available by default)
        for voice in &character_voices {
            sqlx::query("INSERT INTO hero_voices (hero_uid, voice_id) VALUES (?, ?)")
                .bind(hero_uid)
                .bind(voice.audio)
                .execute(&mut **tx)
                .await?;
        }

        // Get all NON-DEFAULT skins for this character from skin table
        for skin in game_data
            .skin
            .iter()
            .filter(|s| s.character_id != 0)
            .filter(|s| s.character_id == hero_id)
            .filter(|s| s.id != hero_skin)
        // Exclude default skin
        {
            // Add non-default skins to hero_skins table (per-hero instance)
            sqlx::query("INSERT INTO hero_skins (hero_uid, skin, expire_sec) VALUES (?, ?, ?)")
                .bind(hero_uid)
                .bind(skin.id)
                .bind(0) // No expiry
                .execute(&mut **tx)
                .await?;

            // Also add to hero_all_skins (account-wide collection)
            sqlx::query("INSERT OR IGNORE INTO hero_all_skins (user_id, skin_id) VALUES (?, ?)")
                .bind(uid)
                .bind(skin.id)
                .execute(&mut **tx)
                .await?;
        }

        // Add item unlocks (from example: [6, 3, 7, 4])
        // These seem to be default unlocked item categories
        // TODO: Determine default item unlocks from excel data
        for item_id in [6, 3, 7, 4] {
            sqlx::query("INSERT INTO hero_item_unlocks (hero_uid, item_id) VALUES (?, ?)")
                .bind(hero_uid)
                .bind(item_id)
                .execute(&mut **tx)
                .await?;
        }

        // Insert default sp_attrs (all zeros as shown in example)
        sqlx::query(
            r#"
            INSERT INTO hero_sp_attrs (
                hero_uid, revive, heal, absorb, defense_ignore, clutch,
                final_add_dmg, final_drop_dmg, normal_skill_rate, play_add_rate, play_drop_rate,
                dizzy_resistances, sleep_resistances, petrified_resistances, frozen_resistances,
                disarm_resistances, forbid_resistances, seal_resistances, cant_get_exskill_resistances,
                del_ex_point_resistances, stress_up_resistances, control_resilience,
                del_ex_point_resilience, stress_up_resilience, charm_resistances,
                rebound_dmg, extra_dmg, reuse_dmg, big_skill_rate, clutch_dmg
            ) VALUES (
                ?1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            )
            "#,
        )
        .bind(hero_uid)
        .execute(&mut **tx)
        .await?;

        // Birthdays
        sqlx::query(
            "INSERT INTO hero_birthday_info (user_id, hero_id, birthday_count) VALUES (?, ?, ?)",
        )
        .bind(uid)
        .bind(hero_id)
        .bind(1)
        .execute(&mut **tx)
        .await?;

        // Insert destiny stone unlocks from character_destiny table
        if let Some(destiny_data) = game_data
            .character_destiny
            .iter()
            .find(|d| d.hero_id == hero_id)
        {
            // Parse facetsId string (e.g., "300901" or "300301#300302")
            // Split by '#' to get all stone IDs
            for stone_str in destiny_data.facets_id.split('#') {
                if let Ok(stone_id) = stone_str.parse::<i32>() {
                    sqlx::query(
                        "INSERT INTO hero_destiny_stone_unlocks (hero_uid, stone_id) VALUES (?, ?)",
                    )
                    .bind(hero_uid)
                    .bind(stone_id)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }

        // Insert talent unlocks from character_talent table
        let char_talent = game_data
            .character_talent
            .iter()
            .filter(|t| t.hero_id == hero_id)
            .max_by_key(|t| t.talent_id);

        if let Some(talent_config) = char_talent {
            // Find the talent scheme
            let talent_scheme = game_data.talent_scheme.iter().find(|s| {
                s.talent_id == talent_config.talent_id
                    && s.talent_mould == talent_config.talent_mould
            });

            if let Some(scheme) = talent_scheme {
                // Parse talenScheme: "10,1,1,0#10,0,0,0#61,1,2,0#..."
                // Format: cubeId,direction,posX,posY
                let cubes: Vec<(i32, i32, i32, i32)> = scheme
                    .talen_scheme
                    .split('#')
                    .filter_map(|cube_str| {
                        let parts: Vec<&str> = cube_str.split(',').collect();
                        if parts.len() == 4 {
                            let cube_id = parts[0].parse::<i32>().ok()?;
                            let direction = parts[1].parse::<i32>().ok()?;
                            let pos_x = parts[2].parse::<i32>().ok()?;
                            let pos_y = parts[3].parse::<i32>().ok()?;
                            Some((cube_id, direction, pos_x, pos_y))
                        } else {
                            None
                        }
                    })
                    .collect();

                // Insert into hero_talent_cubes (active equipped cubes)
                for (cube_id, direction, pos_x, pos_y) in &cubes {
                    sqlx::query(
                        "INSERT INTO hero_talent_cubes (hero_uid, cube_id, direction, pos_x, pos_y) VALUES (?, ?, ?, ?, ?)"
                    )
                    .bind(hero_uid)
                    .bind(cube_id)
                    .bind(direction)
                    .bind(pos_x)
                    .bind(pos_y)
                    .execute(&mut **tx)
                    .await?;
                }

                // Create 4 talent templates (ONLY ONCE, HERE)
                for template_id in 1..=4 {
                    let result = sqlx::query(
                        "INSERT INTO hero_talent_templates (hero_uid, template_id, name, style) VALUES (?, ?, ?, ?)"
                    )
                    .bind(hero_uid)
                    .bind(template_id)
                    .bind("") // Empty name
                    .bind(0) // Style 0
                    .execute(&mut **tx)
                    .await?;

                    let template_row_id = result.last_insert_rowid();

                    // Template #1 gets the same cubes as active (saved preset)
                    // Templates #2-4 are empty
                    if template_id == 1 {
                        for (cube_id, direction, pos_x, pos_y) in &cubes {
                            sqlx::query(
                                "INSERT INTO hero_talent_template_cubes (template_row_id, cube_id, direction, pos_x, pos_y) VALUES (?, ?, ?, ?, ?)"
                            )
                            .bind(template_row_id)
                            .bind(cube_id)
                            .bind(direction)
                            .bind(pos_x)
                            .bind(pos_y)
                            .execute(&mut **tx)
                            .await?;
                        }
                    }
                }

                tracing::debug!(
                    "Loaded {} talent cubes for hero {} (uid {})",
                    cubes.len(),
                    hero_id,
                    hero_uid
                );
            }
        } else {
            // IMPORTANT: Even if hero has no talent data, we still need to create 4 empty templates
            for template_id in 1..=4 {
                sqlx::query(
                    "INSERT INTO hero_talent_templates (hero_uid, template_id, name, style) VALUES (?, ?, ?, ?)"
                )
                .bind(hero_uid)
                .bind(template_id)
                .bind("") // Empty name
                .bind(0) // Style 0
                .execute(&mut **tx)
                .await?;
            }
        }
    }
    // Initialize touch count
    sqlx::query("INSERT INTO hero_touch_count (user_id, touch_count_left) VALUES (?, ?)")
        .bind(uid)
        .bind(5) // Default 5 touches per day
        .execute(&mut **tx)
        .await?;

    // update hero count for player info
    sqlx::query(
        r#"
        UPDATE player_info
        SET
            hero_rare_nn_count  = ?,
            hero_rare_n_count   = ?,
            hero_rare_r_count   = ?,
            hero_rare_sr_count  = ?,
            hero_rare_ssr_count = ?,
            updated_at = ?
        WHERE player_id = ?
        "#,
    )
    .bind(rarity_counts[1]) // NN
    .bind(rarity_counts[2]) // N
    .bind(rarity_counts[3]) // R
    .bind(rarity_counts[4]) // SR
    .bind(rarity_counts[5]) // SSR
    .bind(now)
    .bind(uid)
    .execute(&mut **tx)
    .await?;

    tracing::info!(
        "Generated {} level 1 starter heroes from excel data for uid {}",
        characters.len(),
        uid
    );
    Ok(())
}

/// Load equipment from equip table
pub async fn load_equipment(
    tx: &mut Transaction<'_, Sqlite>,
    uid: i64,
) -> sqlx::Result<HashMap<i32, i64>> {
    let game_data = exceldb::get();
    let now = common::time::ServerTime::now_ms();

    // Starting seed for equipment UIDs
    static EQUIP_UID_COUNTER: AtomicI64 = AtomicI64::new(30000000);

    // Map to store equip_id -> equipment_uid
    let mut equip_map: HashMap<i32, i64> = HashMap::new();

    // Get all equipment (filter out exp items if needed)
    let equipment_list: Vec<_> = game_data
        .equip
        .iter()
        .filter(|e| e.rare == 5 || e.rare == 4 || e.rare == 3 || e.rare == 2)
        .collect();

    for equip in equipment_list.clone() {
        let equip_uid = EQUIP_UID_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Determine max level and break level based on rarity
        let (max_level, max_break_lv, max_refine_lv, is_locked) = match equip.rare {
            5 => (60, 3, 5, true),  // SSR: Level 60, Break 3, Refine 5, LOCKED
            4 => (60, 3, 5, true),  // SR: LOCKED
            3 => (60, 3, 5, false), // R: Not locked
            2 => (60, 3, 5, false), // N: Not locked
            1 => (60, 3, 5, false), // Common: Not locked
            _ => (60, 3, 5, false),
        };

        sqlx::query(
            r#"
            INSERT INTO equipment (
                uid, user_id, equip_id, level, exp, break_lv, count, is_lock, refine_lv, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(equip_uid)
        .bind(uid)
        .bind(equip.id)
        .bind(max_level)
        .bind(0) // exp
        .bind(max_break_lv)
        .bind(1) // count
        .bind(is_locked)
        .bind(max_refine_lv)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        // Store the mapping (equip_id -> first created uid for this equip)
        equip_map.entry(equip.id).or_insert(equip_uid);
    }

    tracing::info!("Loaded {} equipment for uid {}", equipment_list.len(), uid);
    Ok(equip_map)
}

pub async fn load_starter_items(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let game_data = exceldb::get();
    let now = common::time::ServerTime::now_ms();

    static POWER_ITEM_UID_COUNTER: AtomicI64 = AtomicI64::new(40000000);
    static INSIGHT_ITEM_UID_COUNTER: AtomicI64 = AtomicI64::new(50000000);

    // Load stackable items
    let stackable_items: Vec<_> = game_data
        .item
        .iter()
        .filter(|i| i.is_show == 1 && i.is_stackable == 1)
        .filter(|i| i.expire_time == "".to_string())
        .collect();

    for item in stackable_items.clone() {
        let quantity = match item.sub_type {
            _ => 100, // Default
        };

        sqlx::query(
            r#"
            INSERT INTO items (
                user_id, item_id, quantity, last_use_time, last_update_time, total_gain_count
            ) VALUES (?1, ?2, ?3, NULL, ?4, ?5)
            "#,
        )
        .bind(user_id)
        .bind(item.id)
        .bind(quantity)
        .bind(now)
        .bind(quantity as i64)
        .execute(&mut **tx)
        .await?;
    }

    // Load non-stackable items (quantity = 1 always)
    let non_stackable_items: Vec<_> = game_data
        .item
        .iter()
        .filter(|i| i.is_show == 1 && i.is_stackable == 0)
        .filter(|i| i.expire_time == "".to_string())
        .collect();

    for item in non_stackable_items.clone() {
        sqlx::query(
            r#"
            INSERT INTO items (
                user_id, item_id, quantity, last_use_time, last_update_time, total_gain_count
            ) VALUES (?1, ?2, ?3, NULL, ?4, ?5)
            "#,
        )
        .bind(user_id)
        .bind(item.id)
        .bind(1) // Always 1 for non-stackable
        .bind(now)
        .bind(1i64)
        .execute(&mut **tx)
        .await?;
    }

    // Load power items
    let power_items: Vec<_> = game_data
        .power_item
        .iter()
        .filter(|p| p.rare >= 3)
        .collect();

    for power_item in power_items.clone() {
        let uid = POWER_ITEM_UID_COUNTER.fetch_add(1, Ordering::SeqCst);

        let expire_time = match power_item.expire_type {
            1 => {
                //let hours: i64 = power_item.expire_time.parse().unwrap_or(24);
                //now + (hours * 3600)
                0
            }
            _ => 0, //now + 86400,
        };

        sqlx::query(
            r#"
            INSERT INTO power_items (
                uid, user_id, item_id, quantity, expire_time, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(uid)
        .bind(user_id)
        .bind(power_item.id)
        .bind(1)
        .bind(expire_time as i32)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }

    // Load insight items
    let insight_items: Vec<_> = game_data
        .insight_item
        .iter()
        .filter(|i| i.rare >= 4)
        .collect();

    for insight_item in insight_items.clone() {
        let uid = INSIGHT_ITEM_UID_COUNTER.fetch_add(1, Ordering::SeqCst);

        let expire_time = now + 10 * 24 * 3600;
        sqlx::query(
            r#"
            INSERT INTO insight_items (
                uid, user_id, item_id, quantity, expire_time, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(uid)
        .bind(user_id)
        .bind(insight_item.id)
        .bind(1)
        .bind(expire_time as i32)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }

    tracing::info!(
        "Loaded {} stackable items, {} non-stackable items, {} power items, {} insight items for user {}",
        stackable_items.len(),
        non_stackable_items.len(),
        power_items.len(),
        insight_items.len(),
        user_id
    );

    Ok(())
}

pub async fn load_starter_currencies(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let game_data = exceldb::get();
    let now = common::time::ServerTime::now_ms();

    let currencies: Vec<_> = game_data.currency.iter().collect();

    // Load all currencies from excel data
    for currency in currencies.clone() {
        let mut quantity = if currency.max_limit != 0 {
            3_000_000
        } else {
            100
        };

        if currency.id == 4 {
            quantity = 240;
        }

        sqlx::query(
            "INSERT INTO currencies (user_id, currency_id, quantity, last_recover_time, expired_time)
             VALUES (?, ?, ?, ?, 0)"
        )
        .bind(user_id)
        .bind(currency.id)
        .bind(quantity)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }

    tracing::info!(
        "Loaded {} currencies for user {}",
        game_data.currency.len(),
        user_id
    );

    Ok(())
}

pub async fn load_starter_guides(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let game_data = exceldb::get();

    // Load all guides from excel data, set to completed (step_id = -1)
    let guides: Vec<_> = game_data
        .guide
        .iter()
        .filter(|g| g.is_online == 1) // Only load active guides
        .collect();

    for guide in guides {
        sqlx::query(
            "INSERT INTO guide_progress (user_id, guide_id, step_id)
             VALUES (?, ?, -1)",
        )
        .bind(user_id)
        .bind(guide.id)
        .execute(&mut **tx)
        .await?;
    }

    tracing::info!(
        "Loaded {} guides for user {}",
        game_data.guide.len(),
        user_id
    );

    Ok(())
}

pub async fn load_starter_user_stats(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO user_stats (user_id, first_charge, total_charge_amount, is_first_login, user_tag)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(false)
    .bind(590614)
    .bind(false)
    .bind("用户类型7")
    .execute(&mut **tx)
    .await?;

    tracing::info!("Loaded starter user stats for user {}", user_id);

    Ok(())
}

pub async fn load_starter_hero_groups(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();

    // Load hero_group_common_list.json
    let common_json = include_str!("../../../data/static/heros/hero_group_common_list.json");
    let common_data: Value = match serde_json::from_str(common_json) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "load_starter_hero_groups: failed to parse common JSON: {}",
                e
            );
            return Ok(());
        }
    };

    // Load common groups
    if let Some(groups) = common_data
        .get("heroGroupCommons")
        .and_then(|v| v.as_array())
    {
        for group in groups {
            let group_id = group.get("groupId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let name = group.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let cloth_id = group.get("clothId").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
            let assist_boss_id = group
                .get("assistBossId")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            // Insert group
            let group_result = sqlx::query(
                "INSERT INTO hero_groups_common (user_id, group_id, name, cloth_id, assist_boss_id, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(user_id)
            .bind(group_id)
            .bind(name)
            .bind(cloth_id)
            .bind(assist_boss_id)
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;

            let db_group_id = group_result.last_insert_rowid();

            // Add heroes
            if let Some(hero_list) = group.get("heroList").and_then(|v| v.as_array()) {
                for (position, hero_uid) in hero_list.iter().enumerate() {
                    if let Some(uid) = hero_uid.as_i64() {
                        sqlx::query(
                            "INSERT INTO hero_group_members (hero_group_id, hero_uid, position) VALUES (?, ?, ?)"
                        )
                        .bind(db_group_id)
                        .bind(uid)
                        .bind(position as i32)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }

            // Add equips
            if let Some(equips) = group.get("equips").and_then(|v| v.as_array()) {
                for equip in equips {
                    let index = equip.get("index").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    if let Some(equip_uids) = equip.get("equipUid").and_then(|v| v.as_array()) {
                        for equip_uid in equip_uids {
                            if let Some(uid) = equip_uid.as_i64() {
                                sqlx::query(
                                    "INSERT INTO hero_group_equips (hero_group_id, index_slot, equip_uid) VALUES (?, ?, ?)"
                                )
                                .bind(db_group_id)
                                .bind(index)
                                .bind(uid)
                                .execute(&mut **tx)
                                .await?;
                            }
                        }
                    }
                }
            }

            // Add activity104 equips
            if let Some(equips) = group.get("activity104Equips").and_then(|v| v.as_array()) {
                for equip in equips {
                    let index = equip.get("index").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    if let Some(equip_uids) = equip.get("equipUid").and_then(|v| v.as_array()) {
                        for equip_uid in equip_uids {
                            if let Some(uid) = equip_uid.as_i64() {
                                sqlx::query(
                                    "INSERT INTO hero_group_activity104_equips (hero_group_id, index_slot, equip_uid) VALUES (?, ?, ?)"
                                )
                                .bind(db_group_id)
                                .bind(index)
                                .bind(uid)
                                .execute(&mut **tx)
                                .await?;
                            }
                        }
                    }
                }
            }
        }
    }

    // Load hero group types
    if let Some(types) = common_data.get("heroGourpTypes").and_then(|v| v.as_array()) {
        for type_info in types {
            let type_id = type_info.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let current_select = type_info
                .get("currentSelect")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            // Get group_id from groupInfo if present
            let group_id = type_info
                .get("groupInfo")
                .and_then(|info| info.get("groupId"))
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);

            sqlx::query(
                "INSERT INTO hero_group_types (user_id, type_id, current_select, group_id, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(user_id)
            .bind(type_id)
            .bind(current_select)
            .bind(group_id)
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;
        }
    }

    tracing::info!("Loaded hero groups from JSON for user {}", user_id);
    Ok(())
}

pub async fn load_hero_group_snapshots(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();

    let json_str = include_str!("../../../data/static/heros/hero_group_snapshots.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_hero_group_snapshots: failed to parse JSON: {}", e);
            return Ok(());
        }
    };

    let snapshots = match data.get("heroGroupSnapshots").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => {
            eprintln!("load_hero_group_snapshots: missing 'heroGroupSnapshots' array");
            return Ok(());
        }
    };

    for snapshot in snapshots {
        let snapshot_id = snapshot
            .get("snapshotId")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        // Insert main snapshot
        sqlx::query(
            "INSERT INTO hero_group_snapshots (user_id, snapshot_id, created_at, updated_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(snapshot_id)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;

        // Get the DB snapshot ID
        let db_snapshot_id: i64 = sqlx::query_scalar(
            "SELECT id FROM hero_group_snapshots WHERE user_id = ? AND snapshot_id = ?",
        )
        .bind(user_id)
        .bind(snapshot_id)
        .fetch_one(&mut **tx)
        .await?;

        // Insert each group in the snapshot
        if let Some(groups) = snapshot
            .get("heroGroupSnapshots")
            .and_then(|v| v.as_array())
        {
            for group in groups {
                let group_id = group.get("groupId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let name = group.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let cloth_id = group.get("clothId").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                let assist_boss_id = group
                    .get("assistBossId")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;

                let group_result = sqlx::query(
                    "INSERT INTO hero_group_snapshot_groups (snapshot_id, group_id, name, cloth_id, assist_boss_id)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(db_snapshot_id)
                .bind(group_id)
                .bind(name)
                .bind(cloth_id)
                .bind(assist_boss_id)
                .execute(&mut **tx)
                .await?;

                let snapshot_group_id = group_result.last_insert_rowid();

                // Insert heroes
                if let Some(hero_list) = group.get("heroList").and_then(|v| v.as_array()) {
                    for (position, hero_uid) in hero_list.iter().enumerate() {
                        if let Some(uid) = hero_uid.as_i64() {
                            sqlx::query(
                                "INSERT INTO hero_group_snapshot_members (snapshot_group_id, hero_uid, position)
                                 VALUES (?, ?, ?)"
                            )
                            .bind(snapshot_group_id)
                            .bind(uid)
                            .bind(position as i32)
                            .execute(&mut **tx)
                            .await?;
                        }
                    }
                }

                // Insert equips
                if let Some(equips) = group.get("equips").and_then(|v| v.as_array()) {
                    for equip in equips {
                        let index = equip.get("index").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        if let Some(equip_uids) = equip.get("equipUid").and_then(|v| v.as_array()) {
                            for equip_uid in equip_uids {
                                if let Some(uid) = equip_uid.as_i64() {
                                    sqlx::query(
                                        "INSERT INTO hero_group_snapshot_equips (snapshot_group_id, index_slot, equip_uid)
                                         VALUES (?, ?, ?)"
                                    )
                                    .bind(snapshot_group_id)
                                    .bind(index)
                                    .bind(uid)
                                    .execute(&mut **tx)
                                    .await?;
                                }
                            }
                        }
                    }
                }

                // Insert activity104 equips
                if let Some(equips) = group.get("activity104Equips").and_then(|v| v.as_array()) {
                    for equip in equips {
                        let index = equip.get("index").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                        if let Some(equip_uids) = equip.get("equipUid").and_then(|v| v.as_array()) {
                            for equip_uid in equip_uids {
                                if let Some(uid) = equip_uid.as_i64() {
                                    sqlx::query(
                                        "INSERT INTO hero_group_snapshot_activity104_equips (snapshot_group_id, index_slot, equip_uid)
                                         VALUES (?, ?, ?)"
                                    )
                                    .bind(snapshot_group_id)
                                    .bind(index)
                                    .bind(uid)
                                    .execute(&mut **tx)
                                    .await?;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Insert sort IDs
        if let Some(sort_ids) = snapshot.get("sortSubIds").and_then(|v| v.as_array()) {
            for (order, sub_id) in sort_ids.iter().enumerate() {
                if let Some(id) = sub_id.as_i64() {
                    sqlx::query(
                        "INSERT INTO hero_group_snapshot_sort_ids (snapshot_id, sub_id, sort_order)
                         VALUES (?, ?, ?)",
                    )
                    .bind(db_snapshot_id)
                    .bind(id as i32)
                    .bind(order as i32)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
    }

    tracing::info!(
        "Loaded {} hero group snapshots for user {}",
        snapshots.len(),
        user_id
    );
    Ok(())
}

/// Load dungeon starter data from dungeon.json
pub async fn load_dungeon_info(tx: &mut Transaction<'_, Sqlite>, user_id: i64) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/dungeon.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_dungeon_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let now = common::time::ServerTime::now_ms();

    // Load dungeon info list
    if let Some(dungeon_list) = data.get("dungeonInfoList").and_then(|v| v.as_array()) {
        for entry in dungeon_list {
            sqlx::query(
                r#"
                INSERT INTO user_dungeons (
                    user_id, chapter_id, episode_id, star, challenge_count,
                    has_record, left_return_all_num, today_pass_num, today_total_num,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(entry.get("chapterId").and_then(|v| v.as_i64()))
            .bind(entry.get("episodeId").and_then(|v| v.as_i64()))
            .bind(entry.get("star").and_then(|v| v.as_i64()))
            .bind(entry.get("challengeCount").and_then(|v| v.as_i64()))
            .bind(
                entry
                    .get("hasRecord")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            )
            .bind(entry.get("leftReturnAllNum").and_then(|v| v.as_i64()))
            .bind(entry.get("todayPassNum").and_then(|v| v.as_i64()))
            .bind(entry.get("todayTotalNum").and_then(|v| v.as_i64()))
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;
        }
    }

    // Load last hero groups (store references to existing hero groups)
    if let Some(last_groups) = data.get("lastHeroGroup").and_then(|v| v.as_array()) {
        for entry in last_groups {
            let chapter_id = entry.get("chapterId").and_then(|v| v.as_i64()).unwrap_or(0);

            if let Some(snapshot) = entry.get("heroGroupSnapshot").and_then(|v| v.as_object()) {
                let group_id = snapshot
                    .get("groupId")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1);

                sqlx::query(
                    r#"
                    INSERT INTO dungeon_last_hero_groups (
                        user_id, chapter_id, hero_group_id, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?)
                    "#,
                )
                .bind(user_id)
                .bind(chapter_id as i32)
                .bind(group_id as i32)
                .bind(now)
                .bind(now)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    // Load map IDs
    if let Some(map_ids) = data.get("mapIds").and_then(|v| v.as_array()) {
        for map_id in map_ids {
            if let Some(id) = map_id.as_i64() {
                sqlx::query("INSERT INTO user_dungeon_maps (user_id, map_id) VALUES (?, ?)")
                    .bind(user_id)
                    .bind(id as i32)
                    .execute(&mut **tx)
                    .await?;
            }
        }
    }

    // Load elements
    if let Some(elements) = data.get("elements").and_then(|v| v.as_array()) {
        for element in elements {
            if let Some(id) = element.as_i64() {
                sqlx::query(
                    "INSERT INTO user_dungeon_elements (user_id, element_id, is_finished) VALUES (?, ?, 0)"
                )
                .bind(user_id)
                .bind(id as i32)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    // Load finished elements
    if let Some(finished) = data.get("finishElements").and_then(|v| v.as_array()) {
        for element in finished {
            if let Some(id) = element.as_i64() {
                sqlx::query(
                    "INSERT INTO user_dungeon_elements (user_id, element_id, is_finished) VALUES (?, ?, 1)"
                )
                .bind(user_id)
                .bind(id as i32)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    // Load reward point info
    if let Some(reward_points) = data.get("rewardPointInfo").and_then(|v| v.as_array()) {
        for entry in reward_points {
            let chapter_id = entry.get("chapterId").and_then(|v| v.as_i64()).unwrap_or(0);

            sqlx::query(
                r#"
                INSERT INTO user_dungeon_reward_points (
                    user_id, chapter_id, reward_point, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(chapter_id as i32)
            .bind(entry.get("rewardPoint").and_then(|v| v.as_i64()))
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;

            // Load claimed reward IDs
            if let Some(claimed) = entry.get("hasGetPointRewardIds").and_then(|v| v.as_array()) {
                for reward_id in claimed {
                    if let Some(id) = reward_id.as_i64() {
                        sqlx::query(
                            r#"
                            INSERT INTO user_dungeon_claimed_rewards (
                                user_id, chapter_id, point_reward_id
                            ) VALUES (?, ?, ?)
                            "#,
                        )
                        .bind(user_id)
                        .bind(chapter_id as i32)
                        .bind(id as i32)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }
        }
    }

    // Load equip special chapters
    if let Some(equip_sp) = data.get("equipSpChapters").and_then(|v| v.as_array()) {
        for chapter in equip_sp {
            if let Some(id) = chapter.as_i64() {
                sqlx::query(
                    "INSERT INTO user_dungeon_equip_sp_chapters (user_id, chapter_id) VALUES (?, ?)"
                )
                .bind(user_id)
                .bind(id as i32)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    // Load chapter type nums
    if let Some(chapter_types) = data.get("chapterTypeNums").and_then(|v| v.as_array()) {
        for entry in chapter_types {
            sqlx::query(
                r#"
                INSERT INTO user_chapter_type_nums (
                    user_id, chapter_type, today_pass_num, today_total_num, last_reset_date
                ) VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(entry.get("chapterType").and_then(|v| v.as_i64()))
            .bind(entry.get("todayPassNum").and_then(|v| v.as_i64()))
            .bind(entry.get("todayTotalNum").and_then(|v| v.as_i64()))
            .bind(now)
            .execute(&mut **tx)
            .await?;
        }
    }

    // Load finished puzzles
    if let Some(puzzles) = data.get("finishPuzzles").and_then(|v| v.as_array()) {
        for puzzle in puzzles {
            if let Some(id) = puzzle.as_i64() {
                sqlx::query(
                    "INSERT INTO user_dungeon_finished_puzzles (user_id, puzzle_id) VALUES (?, ?)",
                )
                .bind(user_id)
                .bind(id as i32)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    tracing::info!("Loaded dungeon info for user {}", user_id);
    Ok(())
}

/// Load dungeon info
pub async fn load_dungeon_infos(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();

    let json_files = [
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_1.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_2.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_3.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_4.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_5.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_6.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_7.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_8.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_9.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_10.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_11.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_12.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_13.json"),
        include_str!("../../../data/starter/dungeon/dungeon_infos_push_14.json"),
    ];

    for (i, json_str) in json_files.iter().enumerate() {
        let data: Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "load_dungeon_infos: failed to parse JSON file {}: {}",
                    i + 1,
                    e
                );
                continue;
            }
        };

        if let Some(dungeon_list) = data.get("dungeonInfos").and_then(|v| v.as_array()) {
            for entry in dungeon_list {
                sqlx::query(
                    r#"
                    INSERT INTO user_dungeons (
                        user_id, chapter_id, episode_id, star, challenge_count,
                        has_record, left_return_all_num, today_pass_num, today_total_num,
                        created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(user_id, chapter_id, episode_id) DO UPDATE SET
                        star = excluded.star,
                        challenge_count = excluded.challenge_count,
                        has_record = excluded.has_record,
                        left_return_all_num = excluded.left_return_all_num,
                        today_pass_num = excluded.today_pass_num,
                        today_total_num = excluded.today_total_num,
                        updated_at = excluded.updated_at
                    "#,
                )
                .bind(user_id)
                .bind(entry["chapterId"].as_i64().unwrap())
                .bind(entry["episodeId"].as_i64().unwrap())
                .bind(entry["star"].as_i64().unwrap_or(0))
                .bind(entry["challengeCount"].as_i64().unwrap_or(0))
                .bind(entry["hasRecord"].as_bool().unwrap_or(false))
                .bind(entry["leftReturnAllNum"].as_i64().unwrap_or(0))
                .bind(entry["todayPassNum"].as_i64().unwrap_or(0))
                .bind(entry["todayTotalNum"].as_i64().unwrap_or(0))
                .bind(now)
                .bind(now)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    tracing::info!(
        "Loaded dungeon infos from 14 JSON files for user {}",
        user_id
    );
    Ok(())
}

/// Load story data
pub async fn load_story_data(tx: &mut Transaction<'_, Sqlite>, user_id: i64) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/story/get_story.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_story_data: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let now = common::time::ServerTime::now_ms();

    // Load finished stories
    if let Some(finish_list) = data.get("finishList").and_then(|v| v.as_array()) {
        for story_id in finish_list {
            if let Some(id) = story_id.as_i64() {
                sqlx::query("INSERT INTO user_finished_stories (user_id, story_id) VALUES (?, ?)")
                    .bind(user_id)
                    .bind(id as i32)
                    .execute(&mut **tx)
                    .await?;
            }
        }
    }

    // Load processing stories
    if let Some(processing_list) = data.get("processingList").and_then(|v| v.as_array()) {
        for entry in processing_list {
            sqlx::query(
                r#"
                INSERT INTO user_processing_stories (
                    user_id, story_id, step_id, favor, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(entry.get("storyId").and_then(|v| v.as_i64()))
            .bind(entry.get("stepId").and_then(|v| v.as_i64()))
            .bind(entry.get("favor").and_then(|v| v.as_i64()))
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;
        }
    }

    tracing::info!("Loaded story data for user {}", user_id);
    Ok(())
}

/// Load charge info
pub async fn load_charge_info(tx: &mut Transaction<'_, Sqlite>, user_id: i64) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/charge/get_charge_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_charge_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let now = common::time::ServerTime::now_ms();

    // Load charge infos
    if let Some(infos) = data.get("infos").and_then(|v| v.as_array()) {
        for entry in infos {
            sqlx::query(
                r#"
                INSERT INTO user_charge_info (
                    user_id, charge_id, buy_count, first_charge, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(entry.get("id").and_then(|v| v.as_i64()))
            .bind(entry.get("buyCount").and_then(|v| v.as_i64()))
            .bind(
                entry
                    .get("firstCharge")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            )
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;
        }
    }

    // Load sandbox settings
    let sandbox_enable = data
        .get("sandboxEnable")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let sandbox_balance = data
        .get("sandboxBalance")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    sqlx::query(
        r#"
        INSERT INTO user_sandbox_settings (user_id, sandbox_enable, sandbox_balance)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(sandbox_enable)
    .bind(sandbox_balance as i32)
    .execute(&mut **tx)
    .await?;

    tracing::info!("Loaded charge info for user {}", user_id);
    Ok(())
}

/// Load block package info from block_package_info.json
pub async fn load_block_package_info(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/room/block_package_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_block_package_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    // Load block package IDs
    if let Some(packages) = data.get("blockPackageIds").and_then(|v| v.as_array()) {
        for package_id in packages {
            if let Some(id) = package_id.as_i64() {
                sqlx::query(
                    "INSERT INTO user_block_packages (user_id, block_package_id) VALUES (?, ?)",
                )
                .bind(user_id)
                .bind(id as i32)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    // Load special blocks
    if let Some(special_blocks) = data.get("specialBlocks").and_then(|v| v.as_array()) {
        for entry in special_blocks {
            sqlx::query(
                "INSERT INTO user_special_blocks (user_id, block_id, create_time) VALUES (?, ?, ?)",
            )
            .bind(user_id)
            .bind(entry.get("blockId").and_then(|v| v.as_i64()))
            .bind(entry.get("createTime").and_then(|v| v.as_i64()))
            .execute(&mut **tx)
            .await?;
        }
    }

    tracing::info!("Loaded block package info for user {}", user_id);
    Ok(())
}

/// Load building info from building_info.json
pub async fn load_building_info(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/room/building_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_building_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let now = common::time::ServerTime::now_ms();
    let mut building_uid = 20000000i64; // Starting UID for buildings

    if let Some(building_infos) = data.get("buildingInfos").and_then(|v| v.as_array()) {
        for entry in building_infos {
            sqlx::query(
                r#"
                INSERT INTO user_buildings (
                    uid, user_id, define_id, in_use, x, y, rotate, level, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(building_uid)
            .bind(user_id)
            .bind(entry.get("defineId").and_then(|v| v.as_i64()))
            .bind(entry.get("use").and_then(|v| v.as_bool()).unwrap_or(false))
            .bind(entry.get("x").and_then(|v| v.as_i64()))
            .bind(entry.get("y").and_then(|v| v.as_i64()))
            .bind(entry.get("rotate").and_then(|v| v.as_i64()))
            .bind(entry.get("level").and_then(|v| v.as_i64()))
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;

            building_uid += 1;
        }

        tracing::info!(
            "Loaded {} buildings for user {}",
            building_infos.len(),
            user_id
        );
    }

    Ok(())
}

/// Load character interaction info from character_interaction_info.json
pub async fn load_character_interaction_info(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/room/character_interaction_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_character_interaction_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let now = common::time::ServerTime::now_ms();

    // Load interactions
    if let Some(infos) = data.get("infos").and_then(|v| v.as_array()) {
        for entry in infos {
            let interaction_id = entry.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let is_finished = entry
                .get("finish")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            sqlx::query(
                r#"
                INSERT INTO user_character_interactions (
                    user_id, interaction_id, is_finished, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(interaction_id as i32)
            .bind(is_finished)
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;

            // Load select IDs
            if let Some(select_ids) = entry.get("selectIds").and_then(|v| v.as_array()) {
                for select_id in select_ids {
                    if let Some(id) = select_id.as_i64() {
                        sqlx::query(
                            r#"
                            INSERT INTO user_character_interaction_selections (
                                user_id, interaction_id, select_id
                            ) VALUES (?, ?, ?)
                            "#,
                        )
                        .bind(user_id)
                        .bind(interaction_id as i32)
                        .bind(id as i32)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }
        }
    }

    // Load interaction count
    let interaction_count = data
        .get("interactionCount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    sqlx::query("INSERT INTO user_interaction_stats (user_id, interaction_count) VALUES (?, ?)")
        .bind(user_id)
        .bind(interaction_count as i32)
        .execute(&mut **tx)
        .await?;

    tracing::info!("Loaded character interaction info for user {}", user_id);
    Ok(())
}

pub async fn load_summon_info(tx: &mut Transaction<'_, Sqlite>, user_id: i64) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/summon/summon_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_summon_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let now = common::time::ServerTime::now_ms();

    // Load main summon stats
    sqlx::query(
        r#"
        INSERT INTO user_summon_stats (
            user_id, free_equip_summon, is_show_new_summon, new_summon_count, total_summon_count
        ) VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(
        data.get("freeEquipSummon")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    )
    .bind(
        data.get("isShowNewSummon")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    )
    .bind(
        data.get("newSummonCount")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
    )
    .bind(
        data.get("totalSummonCount")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
    )
    .execute(&mut **tx)
    .await?;

    // Load pool infos
    if let Some(pool_infos) = data.get("poolInfos").and_then(|v| v.as_array()) {
        for pool_entry in pool_infos {
            let pool_id = pool_entry
                .get("poolId")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            // Insert pool
            sqlx::query(
                r#"
                INSERT INTO user_summon_pools (
                    user_id, pool_id, online_time, offline_time, have_free, used_free_count,
                    discount_time, can_get_guarantee_sr_count, guarantee_sr_countdown, summon_count,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(pool_id)
            .bind(
                pool_entry
                    .get("onlineTime")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                pool_entry
                    .get("offlineTime")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                pool_entry
                    .get("haveFree")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            )
            .bind(
                pool_entry
                    .get("usedFreeCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                pool_entry
                    .get("discountTime")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                pool_entry
                    .get("canGetGuaranteeSRCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                pool_entry
                    .get("guaranteeSRCountDown")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                pool_entry
                    .get("summonCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;

            // Load lucky bag info
            if let Some(lucky_bag) = pool_entry.get("luckyBagInfo").and_then(|v| v.as_object()) {
                sqlx::query(
                    r#"
                    INSERT INTO user_lucky_bags (user_id, pool_id, count, not_ssr_count)
                    VALUES (?, ?, ?, ?)
                    "#,
                )
                .bind(user_id)
                .bind(pool_id)
                .bind(lucky_bag.get("count").and_then(|v| v.as_i64()).unwrap_or(0))
                .bind(
                    lucky_bag
                        .get("notSSRCount")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                )
                .execute(&mut **tx)
                .await?;

                // Load single bag infos
                if let Some(single_bags) =
                    lucky_bag.get("singleBagInfos").and_then(|v| v.as_array())
                {
                    for bag_entry in single_bags {
                        sqlx::query(
                            r#"
                            INSERT INTO user_single_bags (user_id, pool_id, bag_id, is_open)
                            VALUES (?, ?, ?, ?)
                            "#,
                        )
                        .bind(user_id)
                        .bind(pool_id)
                        .bind(bag_entry.get("bagId").and_then(|v| v.as_i64()).unwrap_or(0))
                        .bind(
                            bag_entry
                                .get("isOpen")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        )
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }

            // Load sp pool info
            if let Some(sp_pool) = pool_entry.get("spPoolInfo").and_then(|v| v.as_object()) {
                sqlx::query(
                    r#"
                    INSERT INTO user_sp_pool_info (
                        user_id, pool_id, sp_type, limited_ticket_id, limited_ticket_num,
                        open_time, used_first_ssr_guarantee
                    ) VALUES (?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(user_id)
                .bind(pool_id)
                .bind(sp_pool.get("type").and_then(|v| v.as_i64()).unwrap_or(0))
                .bind(
                    sp_pool
                        .get("limitedTicketId")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                )
                .bind(
                    sp_pool
                        .get("limitedTicketNum")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                )
                .bind(
                    sp_pool
                        .get("openTime")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                )
                .bind(
                    sp_pool
                        .get("usedFirstSSRGuarantee")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                )
                .execute(&mut **tx)
                .await?;

                // Load up hero IDs
                if let Some(up_hero_ids) = sp_pool.get("UpHeroIds").and_then(|v| v.as_array()) {
                    for hero_id in up_hero_ids {
                        if let Some(id) = hero_id.as_i64() {
                            sqlx::query(
                                r#"
                                INSERT INTO user_sp_pool_up_heroes (user_id, pool_id, hero_id)
                                VALUES (?, ?, ?)
                                "#,
                            )
                            .bind(user_id)
                            .bind(pool_id)
                            .bind(id as i32)
                            .execute(&mut **tx)
                            .await?;
                        }
                    }
                }

                // Load reward progresses
                if let Some(progresses) = sp_pool
                    .get("hasGetRewardProgresses")
                    .and_then(|v| v.as_array())
                {
                    for progress_id in progresses {
                        if let Some(id) = progress_id.as_i64() {
                            sqlx::query(
                                r#"
                                INSERT INTO user_sp_pool_reward_progress (user_id, pool_id, progress_id)
                                VALUES (?, ?, ?)
                                "#
                            )
                            .bind(user_id)
                            .bind(pool_id)
                            .bind(id as i32)
                            .execute(&mut **tx)
                            .await?;
                        }
                    }
                }
            }
        }
    }

    tracing::info!("Loaded summon info for user {}", user_id);
    Ok(())
}

fn parse_datetime_to_unix_sec(s: &str) -> i64 {
    use chrono::{NaiveDateTime, TimeZone, Utc};

    match NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        Ok(dt) => Utc.from_utc_datetime(&dt).timestamp(),
        Err(_) => 0,
    }
}

pub async fn load_summon_history(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/summon/summon_history.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_summon_history: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let Some(page_data) = data
        .get("data")
        .and_then(|v| v.get("pageData"))
        .and_then(|v| v.as_array())
    else {
        tracing::warn!("load_summon_history: no pageData");
        return Ok(());
    };

    for entry in page_data {
        let pool_id = entry.get("poolId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let pool_type = entry.get("poolType").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let pool_name = entry
            .get("poolName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let summon_type = entry
            .get("summonType")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        let summon_time = entry
            .get("createTime")
            .and_then(|v| v.as_str())
            .map(parse_datetime_to_unix_sec)
            .unwrap_or(0);

        // Insert summon history row
        let history_id: i64 = sqlx::query_scalar(
            r#"
            INSERT OR IGNORE INTO user_summon_history (
                user_id, pool_id, summon_type, pool_type, pool_name, summon_time
            )
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(pool_id)
        .bind(summon_type)
        .bind(pool_type)
        .bind(&pool_name)
        .bind(summon_time)
        .fetch_optional(&mut **tx)
        .await?
        // If ignored (duplicate), fetch existing id
        .unwrap_or_else(|| {
            // fallback query
            0
        });

        if history_id == 0 {
            continue;
        }

        // Insert gained items
        if let Some(gain_ids) = entry.get("gainIds").and_then(|v| v.as_array()) {
            for (idx, gain_id) in gain_ids.iter().enumerate() {
                let Some(gain_id) = gain_id.as_i64() else {
                    continue;
                };

                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO user_summon_history_items (
                        history_id, result_index, gain_id
                    )
                    VALUES (?, ?, ?)
                    "#,
                )
                .bind(history_id)
                .bind(idx as i32)
                .bind(gain_id as i32)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    tracing::info!("Loaded summon history for user {}", user_id);
    Ok(())
}

/// Load achievement info from achievement_info.json
pub async fn load_achievement_info(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/achievement/achievement_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_achievement_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let now = common::time::ServerTime::now_ms();

    if let Some(infos) = data.get("infos").and_then(|v| v.as_array()) {
        for entry in infos {
            sqlx::query(
                r#"
                INSERT INTO user_achievements (
                    user_id, achievement_id, progress, has_finish, is_new, finish_time, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(user_id)
            .bind(entry.get("id").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(entry.get("progress").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(entry.get("hasFinish").and_then(|v| v.as_bool()).unwrap_or(false))
            .bind(entry.get("new").and_then(|v| v.as_bool()).unwrap_or(false))
            .bind(entry.get("finishTime").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await?;
        }
    }

    tracing::info!("Loaded achievement info for user {}", user_id);
    Ok(())
}

/// Load dialog info from dialog_info.json
pub async fn load_dialog_info(tx: &mut Transaction<'_, Sqlite>, user_id: i64) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/dialog/dialog_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_dialog_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    if let Some(dialog_ids) = data.get("dialogIds").and_then(|v| v.as_array()) {
        for dialog_id in dialog_ids {
            if let Some(id) = dialog_id.as_i64() {
                sqlx::query("INSERT INTO user_dialogs (user_id, dialog_id) VALUES (?, ?)")
                    .bind(user_id)
                    .bind(id as i32)
                    .execute(&mut **tx)
                    .await?;
            }
        }
    }

    tracing::info!("Loaded dialog info for user {}", user_id);
    Ok(())
}

/// Load starter antiques from antique table
pub async fn load_starter_antiques(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let game_data = data::exceldb::get();
    let get_time = common::time::ServerTime::now_ms();

    let antiques: Vec<_> = game_data.antique.iter().collect();

    for antique in &antiques.clone() {
        sqlx::query("INSERT INTO user_antiques (user_id, antique_id, get_time) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(antique.id)
            .bind(get_time)
            .execute(&mut **tx)
            .await?;
    }

    tracing::info!("Loaded {} antiques for user {}", antiques.len(), user_id);
    Ok(())
}

/// Load weekwalk info from week_walk_info.json
pub async fn load_weekwalk_info(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/week_walk/week_walk_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_weekwalk_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let time_this_week = data
        .get("timeThisWeek")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // Load main info
    if let Some(info) = data.get("info").and_then(|v| v.as_object()) {
        sqlx::query(
            r#"
            INSERT INTO user_weekwalk_info (
                user_id, time, end_time, max_layer, issue_id,
                is_pop_deep_rule, is_open_deep, is_pop_shallow_settle, is_pop_deep_settle,
                deep_progress, time_this_week
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(info.get("time").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(info.get("endTime").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(info.get("maxLayer").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(info.get("issueId").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(
            info.get("isPopDeepRule")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        )
        .bind(
            info.get("isOpenDeep")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        )
        .bind(
            info.get("isPopShallowSettle")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        )
        .bind(
            info.get("isPopDeepSettle")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        )
        .bind(
            info.get("deepProgress")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        )
        .bind(time_this_week)
        .execute(&mut **tx)
        .await?;

        // Load maps
        if let Some(maps) = info.get("mapInfo").and_then(|v| v.as_array()) {
            for map_entry in maps {
                let map_id = map_entry.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                // Insert map
                sqlx::query(
                    r#"
                    INSERT INTO user_weekwalk_maps (
                        user_id, map_id, scene_id, is_finish, is_finished, buff_id,
                        is_show_buff, is_show_finished, is_show_select_cd
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(user_id)
                .bind(map_id)
                .bind(
                    map_entry
                        .get("sceneId")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                )
                .bind(
                    map_entry
                        .get("isFinish")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                )
                .bind(
                    map_entry
                        .get("isFinished")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                )
                .bind(
                    map_entry
                        .get("buffId")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                )
                .bind(
                    map_entry
                        .get("isShowBuff")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                )
                .bind(
                    map_entry
                        .get("isShowFinished")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                )
                .bind(
                    map_entry
                        .get("isShowSelectCd")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                )
                .execute(&mut **tx)
                .await?;

                // Load battles
                if let Some(battles) = map_entry.get("battleInfos").and_then(|v| v.as_array()) {
                    for battle_entry in battles {
                        let battle_id = battle_entry
                            .get("battleId")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0) as i32;

                        sqlx::query(
                            r#"
                            INSERT INTO user_weekwalk_battles (
                                user_id, map_id, battle_id, star, max_star, hero_group_select, element_id
                            ) VALUES (?, ?, ?, ?, ?, ?, ?)
                            "#
                        )
                        .bind(user_id)
                        .bind(map_id)
                        .bind(battle_id)
                        .bind(battle_entry.get("star").and_then(|v| v.as_i64()).unwrap_or(0))
                        .bind(battle_entry.get("maxStar").and_then(|v| v.as_i64()).unwrap_or(0))
                        .bind(battle_entry.get("heroGroupSelect").and_then(|v| v.as_i64()).unwrap_or(0))
                        .bind(battle_entry.get("elementId").and_then(|v| v.as_i64()).unwrap_or(0))
                        .execute(&mut **tx)
                        .await?;

                        // Load hero IDs for battle
                        if let Some(hero_ids) =
                            battle_entry.get("heroIds").and_then(|v| v.as_array())
                        {
                            for hero_id in hero_ids {
                                if let Some(id) = hero_id.as_i64() {
                                    sqlx::query(
                                        r#"
                                        INSERT INTO user_weekwalk_battle_heroes (
                                            user_id, map_id, battle_id, hero_id
                                        ) VALUES (?, ?, ?, ?)
                                        "#,
                                    )
                                    .bind(user_id)
                                    .bind(map_id)
                                    .bind(battle_id)
                                    .bind(id as i32)
                                    .execute(&mut **tx)
                                    .await?;
                                }
                            }
                        }
                    }
                }

                // Load elements
                if let Some(elements) = map_entry.get("elementInfos").and_then(|v| v.as_array()) {
                    for element_entry in elements {
                        let element_id = element_entry
                            .get("elementId")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0) as i32;

                        sqlx::query(
                            r#"
                            INSERT INTO user_weekwalk_elements (
                                user_id, map_id, element_id, is_finish, index_num, visible
                            ) VALUES (?, ?, ?, ?, ?, ?)
                            "#,
                        )
                        .bind(user_id)
                        .bind(map_id)
                        .bind(element_id)
                        .bind(
                            element_entry
                                .get("isFinish")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        )
                        .bind(
                            element_entry
                                .get("index")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                        )
                        .bind(
                            element_entry
                                .get("visible")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        )
                        .execute(&mut **tx)
                        .await?;

                        // Load history list
                        if let Some(history) =
                            element_entry.get("historylist").and_then(|v| v.as_array())
                        {
                            for (i, history_entry) in history.iter().enumerate() {
                                if let Some(entry_str) = history_entry.as_str() {
                                    sqlx::query(
                                        r#"
                                        INSERT INTO user_weekwalk_element_history (
                                            user_id, map_id, element_id, history_entry, sort_order
                                        ) VALUES (?, ?, ?, ?, ?)
                                        "#,
                                    )
                                    .bind(user_id)
                                    .bind(map_id)
                                    .bind(element_id)
                                    .bind(entry_str)
                                    .bind(i as i32)
                                    .execute(&mut **tx)
                                    .await?;
                                }
                            }
                        }
                    }
                }

                // Load hero infos
                if let Some(heroes) = map_entry.get("heroInfos").and_then(|v| v.as_array()) {
                    for hero_entry in heroes {
                        sqlx::query(
                            r#"
                            INSERT INTO user_weekwalk_heroes (
                                user_id, map_id, hero_id, cd
                            ) VALUES (?, ?, ?, ?)
                            "#,
                        )
                        .bind(user_id)
                        .bind(map_id)
                        .bind(
                            hero_entry
                                .get("heroId")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                        )
                        .bind(hero_entry.get("cd").and_then(|v| v.as_i64()).unwrap_or(0))
                        .execute(&mut **tx)
                        .await?;
                    }
                }

                // Load story IDs
                if let Some(story_ids) = map_entry.get("storyIds").and_then(|v| v.as_array()) {
                    for story_id in story_ids {
                        if let Some(id) = story_id.as_i64() {
                            sqlx::query(
                                r#"
                                INSERT INTO user_weekwalk_stories (
                                    user_id, map_id, story_id
                                ) VALUES (?, ?, ?)
                                "#,
                            )
                            .bind(user_id)
                            .bind(map_id)
                            .bind(id as i32)
                            .execute(&mut **tx)
                            .await?;
                        }
                    }
                }
            }
        }
    }

    tracing::info!("Loaded weekwalk info for user {}", user_id);
    Ok(())
}

/// Load weekwalk v2 info from week_walk_ver2_get_info.json
pub async fn load_weekwalk_v2_info(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/week_walk/week_walk_ver2_get_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_weekwalk_v2_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    if let Some(info) = data.get("info").and_then(|v| v.as_object()) {
        // Insert main info
        sqlx::query(
            r#"
            INSERT INTO user_weekwalk_v2_info (user_id, time_id, start_time, end_time, pop_rule)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(info.get("timeId").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(info.get("startTime").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(info.get("endTime").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(
            info.get("popRule")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        )
        .execute(&mut **tx)
        .await?;

        // Load layer infos
        if let Some(layers) = info.get("layerInfos").and_then(|v| v.as_array()) {
            for layer_entry in layers {
                let layer_id = layer_entry.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

                // Insert layer
                sqlx::query(
                    r#"
                    INSERT INTO user_weekwalk_v2_layers (
                        user_id, layer_id, scene_id, all_pass, finished, unlock, show_finished, params
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    "#
                )
                .bind(user_id)
                .bind(layer_id)
                .bind(layer_entry.get("sceneId").and_then(|v| v.as_i64()).unwrap_or(0))
                .bind(layer_entry.get("allPass").and_then(|v| v.as_bool()).unwrap_or(false))
                .bind(layer_entry.get("finished").and_then(|v| v.as_bool()).unwrap_or(false))
                .bind(layer_entry.get("unlock").and_then(|v| v.as_bool()).unwrap_or(false))
                .bind(layer_entry.get("showFinished").and_then(|v| v.as_bool()).unwrap_or(false))
                .bind(layer_entry.get("params").and_then(|v| v.as_str()).unwrap_or(""))
                .execute(&mut **tx)
                .await?;

                // Load battle infos
                if let Some(battles) = layer_entry.get("battleInfos").and_then(|v| v.as_array()) {
                    for battle_entry in battles {
                        let battle_id = battle_entry
                            .get("battleId")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0) as i32;

                        // Insert battle
                        sqlx::query(
                            r#"
                            INSERT INTO user_weekwalk_v2_battles (
                                user_id, layer_id, battle_id, status, hero_group_select, element_id, params
                            ) VALUES (?, ?, ?, ?, ?, ?, ?)
                            "#
                        )
                        .bind(user_id)
                        .bind(layer_id)
                        .bind(battle_id)
                        .bind(battle_entry.get("status").and_then(|v| v.as_i64()).unwrap_or(0))
                        .bind(battle_entry.get("heroGroupSelect").and_then(|v| v.as_i64()).unwrap_or(0))
                        .bind(battle_entry.get("elementId").and_then(|v| v.as_i64()).unwrap_or(0))
                        .bind(battle_entry.get("params").and_then(|v| v.as_str()).unwrap_or(""))
                        .execute(&mut **tx)
                        .await?;

                        // Load hero IDs
                        if let Some(hero_ids) =
                            battle_entry.get("heroIds").and_then(|v| v.as_array())
                        {
                            for hero_id in hero_ids {
                                if let Some(id) = hero_id.as_i64() {
                                    sqlx::query(
                                        r#"
                                        INSERT INTO user_weekwalk_v2_battle_heroes (
                                            user_id, layer_id, battle_id, hero_id
                                        ) VALUES (?, ?, ?, ?)
                                        "#,
                                    )
                                    .bind(user_id)
                                    .bind(layer_id)
                                    .bind(battle_id)
                                    .bind(id as i32)
                                    .execute(&mut **tx)
                                    .await?;
                                }
                            }
                        }

                        // Load choose skill IDs
                        if let Some(skill_ids) = battle_entry
                            .get("chooseSkillIds")
                            .and_then(|v| v.as_array())
                        {
                            for skill_id in skill_ids {
                                if let Some(id) = skill_id.as_i64() {
                                    sqlx::query(
                                        r#"
                                        INSERT INTO user_weekwalk_v2_battle_skills (
                                            user_id, layer_id, battle_id, skill_id
                                        ) VALUES (?, ?, ?, ?)
                                        "#,
                                    )
                                    .bind(user_id)
                                    .bind(layer_id)
                                    .bind(battle_id)
                                    .bind(id as i32)
                                    .execute(&mut **tx)
                                    .await?;
                                }
                            }
                        }

                        // Load cup infos
                        if let Some(cups) = battle_entry.get("cupInfos").and_then(|v| v.as_array())
                        {
                            for cup_entry in cups {
                                sqlx::query(
                                    r#"
                                    INSERT INTO user_weekwalk_v2_cups (
                                        user_id, layer_id, battle_id, cup_id, result
                                    ) VALUES (?, ?, ?, ?, ?)
                                    "#,
                                )
                                .bind(user_id)
                                .bind(layer_id)
                                .bind(battle_id)
                                .bind(cup_entry.get("id").and_then(|v| v.as_i64()).unwrap_or(0))
                                .bind(
                                    cup_entry
                                        .get("result")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0),
                                )
                                .execute(&mut **tx)
                                .await?;
                            }
                        }
                    }
                }

                // Load element infos
                if let Some(elements) = layer_entry.get("elementInfos").and_then(|v| v.as_array()) {
                    for element_entry in elements {
                        sqlx::query(
                            r#"
                            INSERT INTO user_weekwalk_v2_elements (
                                user_id, layer_id, element_id, finish, index_num, visible
                            ) VALUES (?, ?, ?, ?, ?, ?)
                            "#,
                        )
                        .bind(user_id)
                        .bind(layer_id)
                        .bind(
                            element_entry
                                .get("elementId")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                        )
                        .bind(
                            element_entry
                                .get("finish")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        )
                        .bind(
                            element_entry
                                .get("index")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                        )
                        .bind(
                            element_entry
                                .get("visible")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        )
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }
        }

        // Load previous settle info
        if let Some(prev_settle) = info.get("prevSettle").and_then(|v| v.as_object()) {
            sqlx::query(
                r#"
                INSERT INTO user_weekwalk_v2_prev_settle (
                    user_id, max_layer_id, max_battle_id, max_battle_index, show
                ) VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(
                prev_settle
                    .get("maxLayerId")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                prev_settle
                    .get("maxBattleId")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                prev_settle
                    .get("maxBattleIndex")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                prev_settle
                    .get("show")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            )
            .execute(&mut **tx)
            .await?;

            // Load settle layer infos
            if let Some(settle_layers) = prev_settle.get("layerInfos").and_then(|v| v.as_array()) {
                for settle_layer in settle_layers {
                    sqlx::query(
                        r#"
                        INSERT INTO user_weekwalk_v2_prev_settle_layers (
                            user_id, layer_id, platinum_cup_num
                        ) VALUES (?, ?, ?)
                        "#,
                    )
                    .bind(user_id)
                    .bind(
                        settle_layer
                            .get("layerId")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0),
                    )
                    .bind(
                        settle_layer
                            .get("platinumCupNum")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0),
                    )
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }

        // Load snapshot infos
        if let Some(snapshots) = info.get("snapshotInfos").and_then(|v| v.as_array()) {
            for snapshot_entry in snapshots {
                let snapshot_no = snapshot_entry
                    .get("no")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;

                // Insert snapshot
                sqlx::query(
                    "INSERT INTO user_weekwalk_v2_snapshots (user_id, snapshot_no) VALUES (?, ?)",
                )
                .bind(user_id)
                .bind(snapshot_no)
                .execute(&mut **tx)
                .await?;

                // Load skill IDs
                if let Some(skill_ids) = snapshot_entry.get("skillIds").and_then(|v| v.as_array()) {
                    for skill_id in skill_ids {
                        if let Some(id) = skill_id.as_i64() {
                            sqlx::query(
                                r#"
                                INSERT INTO user_weekwalk_v2_snapshot_skills (
                                    user_id, snapshot_no, skill_id
                                ) VALUES (?, ?, ?)
                                "#,
                            )
                            .bind(user_id)
                            .bind(snapshot_no)
                            .bind(id as i32)
                            .execute(&mut **tx)
                            .await?;
                        }
                    }
                }
            }
        }
    }

    tracing::info!("Loaded weekwalk v2 info for user {}", user_id);
    Ok(())
}

/// Load explore simple info from explore_simple_info.json
pub async fn load_explore_simple_info(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/explore/explore_simple_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_explore_simple_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    // Insert main info
    sqlx::query(
        "INSERT INTO user_explore_info (user_id, last_map_id, is_show_bag) VALUES (?, ?, ?)",
    )
    .bind(user_id)
    .bind(data.get("lastMapId").and_then(|v| v.as_i64()).unwrap_or(0))
    .bind(
        data.get("isShowBag")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    )
    .execute(&mut **tx)
    .await?;

    // Load chapter simple
    if let Some(chapters) = data.get("chapterSimple").and_then(|v| v.as_array()) {
        for chapter_entry in chapters {
            let chapter_id = chapter_entry
                .get("chapterId")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            sqlx::query(
                "INSERT INTO user_explore_chapters (user_id, chapter_id, is_finish) VALUES (?, ?, ?)"
            )
            .bind(user_id)
            .bind(chapter_id)
            .bind(chapter_entry.get("isFinish").and_then(|v| v.as_bool()).unwrap_or(false))
            .execute(&mut **tx)
            .await?;

            // Load archive IDs
            if let Some(archives) = chapter_entry.get("archiveIds").and_then(|v| v.as_array()) {
                for archive_id in archives {
                    if let Some(id) = archive_id.as_i64() {
                        sqlx::query(
                            "INSERT INTO user_explore_chapter_archives (user_id, chapter_id, archive_id) VALUES (?, ?, ?)"
                        )
                        .bind(user_id)
                        .bind(chapter_id)
                        .bind(id as i32)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }

            // Load bonus scenes
            if let Some(bonus_scenes) = chapter_entry.get("bonusScene").and_then(|v| v.as_array()) {
                for bonus_entry in bonus_scenes {
                    let bonus_scene_id = bonus_entry
                        .get("bonusSceneId")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32;

                    sqlx::query(
                        "INSERT INTO user_explore_bonus_scenes (user_id, chapter_id, bonus_scene_id) VALUES (?, ?, ?)"
                    )
                    .bind(user_id)
                    .bind(chapter_id)
                    .bind(bonus_scene_id)
                    .execute(&mut **tx)
                    .await?;

                    // Load options
                    if let Some(options) = bonus_entry.get("options").and_then(|v| v.as_array()) {
                        for option in options {
                            if let Some(opt_id) = option.as_i64() {
                                sqlx::query(
                                    "INSERT INTO user_explore_bonus_scene_options (user_id, chapter_id, bonus_scene_id, option_id) VALUES (?, ?, ?, ?)"
                                )
                                .bind(user_id)
                                .bind(chapter_id)
                                .bind(bonus_scene_id)
                                .bind(opt_id as i32)
                                .execute(&mut **tx)
                                .await?;
                            }
                        }
                    }
                }
            }
        }
    }

    // Load map simple
    if let Some(maps) = data.get("mapSimple").and_then(|v| v.as_array()) {
        for map_entry in maps {
            let map_id = map_entry.get("mapId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            sqlx::query(
                r#"
                INSERT INTO user_explore_maps (
                    user_id, map_id,
                    bonus_num, gold_coin, purple_coin,
                    bonus_num_total, gold_coin_total, purple_coin_total
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(user_id, map_id) DO UPDATE SET
                    bonus_num_total   = excluded.bonus_num_total,
                    gold_coin_total   = excluded.gold_coin_total,
                    purple_coin_total = excluded.purple_coin_total
                "#,
            )
            .bind(user_id)
            .bind(map_id)
            .bind(
                map_entry
                    .get("bonusNum")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                map_entry
                    .get("goldCoin")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                map_entry
                    .get("purpleCoin")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                map_entry
                    .get("bonusNumTotal")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                map_entry
                    .get("goldCoinTotal")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                map_entry
                    .get("purpleCoinTotal")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .execute(&mut **tx)
            .await?;

            // Load bonus IDs
            if let Some(bonus_ids) = map_entry.get("bonusIds").and_then(|v| v.as_array()) {
                for bonus_id in bonus_ids {
                    if let Some(id) = bonus_id.as_i64() {
                        sqlx::query(
                            "INSERT INTO user_explore_map_bonuses (user_id, map_id, bonus_id) VALUES (?, ?, ?)"
                        )
                        .bind(user_id)
                        .bind(map_id)
                        .bind(id as i32)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }
        }
    }

    // Load unlocked map IDs
    if let Some(unlock_maps) = data.get("unlockMapIds").and_then(|v| v.as_array()) {
        for map_id in unlock_maps {
            if let Some(id) = map_id.as_i64() {
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO user_explore_unlocked_maps (user_id, map_id)
                    VALUES (?, ?)
                    "#,
                )
                .bind(user_id)
                .bind(id as i32)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    tracing::info!("Loaded explore simple info for user {}", user_id);
    Ok(())
}

/// Load tower info from tower_info.json
pub async fn load_tower_info(tx: &mut Transaction<'_, Sqlite>, user_id: i64) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/tower/tower_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_tower_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    // Insert main info
    sqlx::query(
        "INSERT INTO user_tower_info (user_id, mop_up_times, trial_hero_season) VALUES (?, ?, ?)",
    )
    .bind(user_id)
    .bind(data.get("mopUpTimes").and_then(|v| v.as_i64()).unwrap_or(0))
    .bind(
        data.get("trialHeroSeason")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
    )
    .execute(&mut **tx)
    .await?;

    // Load tower opens
    if let Some(tower_opens) = data.get("towerOpens").and_then(|v| v.as_array()) {
        for open_entry in tower_opens {
            sqlx::query(
                r#"
                INSERT INTO user_tower_opens (
                    user_id, tower_type, tower_id, status, round, next_time, tower_start_time, task_end_time
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(user_id)
            .bind(open_entry.get("type").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(open_entry.get("towerId").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(open_entry.get("status").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(open_entry.get("round").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(open_entry.get("nextTime").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(open_entry.get("towerStartTime").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(open_entry.get("taskEndTime").and_then(|v| v.as_i64()).unwrap_or(0))
            .execute(&mut **tx)
            .await?;
        }
    }

    // Load towers
    if let Some(towers) = data.get("towers").and_then(|v| v.as_array()) {
        for tower_entry in towers {
            let tower_type = tower_entry
                .get("type")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let tower_id = tower_entry
                .get("towerId")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            // Insert tower
            sqlx::query(
                r#"
                INSERT INTO user_towers (
                    user_id, tower_type, tower_id, pass_layer_id, history_high_score, params
                ) VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(tower_type)
            .bind(tower_id)
            .bind(
                tower_entry
                    .get("passLayerId")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                tower_entry
                    .get("historyHighScore")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                tower_entry
                    .get("params")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            )
            .execute(&mut **tx)
            .await?;

            // Load open special layer IDs
            if let Some(sp_layers) = tower_entry.get("openSpLayerIds").and_then(|v| v.as_array()) {
                for sp_layer_id in sp_layers {
                    if let Some(id) = sp_layer_id.as_i64() {
                        sqlx::query(
                            "INSERT INTO user_tower_open_sp_layers (user_id, tower_type, tower_id, sp_layer_id) VALUES (?, ?, ?, ?)"
                        )
                        .bind(user_id)
                        .bind(tower_type)
                        .bind(tower_id)
                        .bind(id as i32)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }

            // Load pass teach IDs
            if let Some(teach_ids) = tower_entry.get("passTeachIds").and_then(|v| v.as_array()) {
                for teach_id in teach_ids {
                    if let Some(id) = teach_id.as_i64() {
                        sqlx::query(
                            "INSERT INTO user_tower_pass_teaches (user_id, tower_type, tower_id, teach_id) VALUES (?, ?, ?, ?)"
                        )
                        .bind(user_id)
                        .bind(tower_type)
                        .bind(tower_id)
                        .bind(id as i32)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }

            // Load layers
            if let Some(layers) = tower_entry.get("layerNOs").and_then(|v| v.as_array()) {
                for layer_entry in layers {
                    let layer_id = layer_entry
                        .get("layerId")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32;

                    // Insert layer
                    sqlx::query(
                        r#"
                        INSERT INTO user_tower_layers (
                            user_id, tower_type, tower_id, layer_id, curr_high_score, history_high_score
                        ) VALUES (?, ?, ?, ?, ?, ?)
                        "#
                    )
                    .bind(user_id)
                    .bind(tower_type)
                    .bind(tower_id)
                    .bind(layer_id)
                    .bind(layer_entry.get("currHighScore").and_then(|v| v.as_i64()).unwrap_or(0))
                    .bind(layer_entry.get("historyHighScore").and_then(|v| v.as_i64()).unwrap_or(0))
                    .execute(&mut **tx)
                    .await?;

                    // Load episodes
                    if let Some(episodes) = layer_entry.get("episodeNOs").and_then(|v| v.as_array())
                    {
                        for episode_entry in episodes {
                            let episode_id = episode_entry
                                .get("episodeId")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0) as i32;

                            // Insert episode
                            sqlx::query(
                                r#"
                                INSERT INTO user_tower_episodes (
                                    user_id, tower_type, tower_id, layer_id, episode_id, status, assist_boss_id
                                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                                "#
                            )
                            .bind(user_id)
                            .bind(tower_type)
                            .bind(tower_id)
                            .bind(layer_id)
                            .bind(episode_id)
                            .bind(episode_entry.get("status").and_then(|v| v.as_i64()).unwrap_or(0))
                            .bind(episode_entry.get("assistBossId").and_then(|v| v.as_i64()).unwrap_or(0))
                            .execute(&mut **tx)
                            .await?;

                            // Load heroes
                            if let Some(heroes) =
                                episode_entry.get("heros").and_then(|v| v.as_array())
                            {
                                for hero_entry in heroes {
                                    let hero_id = hero_entry
                                        .get("heroId")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0)
                                        as i32;

                                    // Insert hero
                                    sqlx::query(
                                        r#"
                                        INSERT INTO user_tower_episode_heroes (
                                            user_id, tower_type, tower_id, layer_id, episode_id, hero_id, trial_id
                                        ) VALUES (?, ?, ?, ?, ?, ?, ?)
                                        "#
                                    )
                                    .bind(user_id)
                                    .bind(tower_type)
                                    .bind(tower_id)
                                    .bind(layer_id)
                                    .bind(episode_id)
                                    .bind(hero_id)
                                    .bind(hero_entry.get("trialId").and_then(|v| v.as_i64()).unwrap_or(0))
                                    .execute(&mut **tx)
                                    .await?;

                                    // FETCH default_equip_uid FROM heroes TABLE
                                    let default_equip_uid: Option<i64> = sqlx::query_scalar(
                                        "SELECT default_equip_uid FROM heroes WHERE user_id = ? AND hero_id = ?"
                                    )
                                    .bind(user_id)
                                    .bind(hero_id)
                                    .fetch_optional(&mut **tx)
                                    .await?;

                                    // If hero has a default equip, store it
                                    if let Some(equip_uid) = default_equip_uid {
                                        if equip_uid > 0 {
                                            // Only if actually equipped
                                            sqlx::query(
                                                r#"
                                                INSERT INTO user_tower_episode_hero_equips (
                                                    user_id, tower_type, tower_id, layer_id, episode_id, hero_id, equip_uid
                                                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                                                "#
                                            )
                                            .bind(user_id)
                                            .bind(tower_type)
                                            .bind(tower_id)
                                            .bind(layer_id)
                                            .bind(episode_id)
                                            .bind(hero_id)
                                            .bind(equip_uid)
                                            .execute(&mut **tx)
                                            .await?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Load assist bosses (unchanged)
    if let Some(bosses) = data.get("assistBosses").and_then(|v| v.as_array()) {
        for boss_entry in bosses {
            let boss_id = boss_entry.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            sqlx::query(
                r#"
                INSERT INTO user_assist_bosses (user_id, boss_id, level, use_talent_plan)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(boss_id)
            .bind(
                boss_entry
                    .get("level")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1),
            )
            .bind(
                boss_entry
                    .get("useTalentPlan")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .execute(&mut **tx)
            .await?;

            if let Some(plans) = boss_entry.get("talentPlans").and_then(|v| v.as_array()) {
                for plan_entry in plans {
                    let plan_id = plan_entry
                        .get("planId")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32;

                    sqlx::query(
                        r#"
                        INSERT INTO user_assist_boss_talent_plans (
                            user_id, boss_id, plan_id, talent_point, plan_name
                        ) VALUES (?, ?, ?, ?, ?)
                        "#,
                    )
                    .bind(user_id)
                    .bind(boss_id)
                    .bind(plan_id)
                    .bind(
                        plan_entry
                            .get("talentPoint")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0),
                    )
                    .bind(
                        plan_entry
                            .get("planName")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    )
                    .execute(&mut **tx)
                    .await?;

                    if let Some(talents) = plan_entry.get("talentIds").and_then(|v| v.as_array()) {
                        for talent_id in talents {
                            if let Some(id) = talent_id.as_i64() {
                                sqlx::query(
                                    r#"
                                    INSERT INTO user_assist_boss_plan_talents (
                                        user_id, boss_id, plan_id, talent_id
                                    ) VALUES (?, ?, ?, ?)
                                    "#,
                                )
                                .bind(user_id)
                                .bind(boss_id)
                                .bind(plan_id)
                                .bind(id as i32)
                                .execute(&mut **tx)
                                .await?;
                            }
                        }
                    }
                }
            }
        }
    }

    tracing::info!("Loaded tower info for user {}", user_id);
    Ok(())
}

/// Load player card info from player_card_info.json
pub async fn load_player_card_info(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/player_card/player_card_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_player_card_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    if let Some(card_info) = data.get("playerCardInfo").and_then(|v| v.as_object()) {
        // Convert showSettings array to JSON string
        let show_settings = card_info
            .get("showSettings")
            .and_then(|v| serde_json::to_string(v).ok())
            .unwrap_or_else(|| "[]".to_string());

        sqlx::query(
            r#"
            INSERT INTO user_player_card_info (
                user_id, show_settings, progress_setting, base_setting, hero_cover, theme_id,
                show_achievement, critter, room_collection, weekwalk_deep_layer_id, explore_collection,
                rouge_difficulty, act128_sss_count, achievement_count, assist_times, hero_cover_times,
                max_faith_hero_count, total_cost_power, skin_count, tower_layer, tower_boss_pass_count,
                hero_max_level_count, weekwalk_ver2_platinum_cup, hero_count, tower_layer_metre
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(user_id)
        .bind(show_settings)
        .bind(card_info.get("progressSetting").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(card_info.get("baseSetting").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(card_info.get("heroCover").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(card_info.get("themeId").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("showAchievement").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(card_info.get("critter").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(card_info.get("roomCollection").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(card_info.get("weekwalkDeepLayerId").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("exploreCollection").and_then(|v| v.as_str()).unwrap_or(""))
        .bind(card_info.get("rougeDifficulty").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("act128SSSCount").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("achievementCount").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("assistTimes").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("heroCoverTimes").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("maxFaithHeroCount").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("totalCostPower").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("skinCount").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("towerLayer").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("towerBossPassCount").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("heroMaxLevelCount").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("weekwalkVer2PlatinumCup").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("heroCount").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(card_info.get("towerLayerMetre").and_then(|v| v.as_i64()).unwrap_or(0))
        .execute(&mut **tx)
        .await?;
    }

    tracing::info!("Loaded player card info for user {}", user_id);
    Ok(())
}

/// Load command post info from command_post_info.json
pub async fn load_command_post_info(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/command/command_post_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_command_post_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    // Insert main info
    sqlx::query("INSERT INTO user_command_post_info (user_id, paper, catch_num) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(data.get("paper").and_then(|v| v.as_i64()).unwrap_or(0))
        .bind(data.get("catchNum").and_then(|v| v.as_i64()).unwrap_or(0))
        .execute(&mut **tx)
        .await?;

    // Load events
    if let Some(events) = data.get("eventList").and_then(|v| v.as_array()) {
        for event_entry in events {
            let event_id = event_entry.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            sqlx::query(
                r#"
                INSERT INTO user_command_post_events (
                    user_id, event_id, state, start_time, end_time, is_read
                ) VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(event_id)
            .bind(
                event_entry
                    .get("state")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                event_entry
                    .get("startTime")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                event_entry
                    .get("endTime")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                event_entry
                    .get("read")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            )
            .execute(&mut **tx)
            .await?;

            // Load hero IDs
            if let Some(hero_ids) = event_entry.get("heroIds").and_then(|v| v.as_array()) {
                for hero_id in hero_ids {
                    if let Some(id) = hero_id.as_i64() {
                        sqlx::query(
                            "INSERT INTO user_command_post_event_heroes (user_id, event_id, hero_id) VALUES (?, ?, ?)"
                        )
                        .bind(user_id)
                        .bind(event_id)
                        .bind(id as i32)
                        .execute(&mut **tx)
                        .await?;
                    }
                }
            }
        }
    }

    // Load tasks
    if let Some(tasks) = data.get("tasks").and_then(|v| v.as_array()) {
        for task_entry in tasks {
            sqlx::query(
                r#"
                INSERT INTO user_command_post_tasks (
                    user_id, task_id, progress, has_finished, finish_count, task_type, expiry_time
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(task_entry.get("id").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(
                task_entry
                    .get("progress")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                task_entry
                    .get("hasFinished")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            )
            .bind(
                task_entry
                    .get("finishCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(task_entry.get("type").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(
                task_entry
                    .get("expiryTime")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .execute(&mut **tx)
            .await?;
        }
    }

    // Load catch tasks
    if let Some(catch_tasks) = data.get("catchTasks").and_then(|v| v.as_array()) {
        for task_entry in catch_tasks {
            sqlx::query(
                r#"
                INSERT INTO user_command_post_catch_tasks (
                    user_id, task_id, progress, has_finished, finish_count, task_type, expiry_time
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(task_entry.get("id").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(
                task_entry
                    .get("progress")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(
                task_entry
                    .get("hasFinished")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            )
            .bind(
                task_entry
                    .get("finishCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .bind(task_entry.get("type").and_then(|v| v.as_i64()).unwrap_or(0))
            .bind(
                task_entry
                    .get("expiryTime")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            )
            .execute(&mut **tx)
            .await?;
        }
    }

    // Load gain bonuses
    if let Some(bonuses) = data.get("gainBonus").and_then(|v| v.as_array()) {
        for bonus_id in bonuses {
            if let Some(id) = bonus_id.as_i64() {
                sqlx::query(
                    "INSERT INTO user_command_post_gain_bonus (user_id, bonus_id) VALUES (?, ?)",
                )
                .bind(user_id)
                .bind(id as i32)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    tracing::info!("Loaded command post info for user {}", user_id);
    Ok(())
}

/// Load friend info from friend_info.json
pub async fn load_friend_info(tx: &mut Transaction<'_, Sqlite>, user_id: i64) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/friend/friend_info.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_friend_info: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    let now = common::time::ServerTime::now_ms();

    // Load friend IDs
    if let Some(friend_ids) = data.get("friendIds").and_then(|v| v.as_array()) {
        for friend_id in friend_ids {
            if let Some(id) = friend_id.as_u64() {
                sqlx::query(
                    "INSERT INTO user_friends (user_id, friend_id, created_at) VALUES (?, ?, ?)",
                )
                .bind(user_id)
                .bind(id as i64)
                .bind(now)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    // Load blacklist IDs
    if let Some(blacklist_ids) = data.get("blackListIds").and_then(|v| v.as_array()) {
        for blocked_id in blacklist_ids {
            if let Some(id) = blocked_id.as_u64() {
                sqlx::query(
                    "INSERT INTO user_blacklist (user_id, blocked_user_id, created_at) VALUES (?, ?, ?)"
                )
                .bind(user_id)
                .bind(id as i64)
                .bind(now)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    tracing::info!("Loaded friend info for user {}", user_id);
    Ok(())
}

/// Load simple properties from simple_property.json
pub async fn load_simple_properties(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/starter/property/simple_property.json");
    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load_simple_properties: failed to parse JSON: {e}");
            return Ok(());
        }
    };

    if let Some(properties) = data.get("simpleProperties").and_then(|v| v.as_array()) {
        for prop in properties {
            let property_id = prop.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let property_value = prop
                .get("property")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            sqlx::query(
                "INSERT INTO user_simple_properties (user_id, property_id, property_value) VALUES (?, ?, ?)"
            )
            .bind(user_id)
            .bind(property_id)
            .bind(property_value)
            .execute(&mut **tx)
            .await?;
        }
    }

    tracing::info!("Loaded simple properties for user {}", user_id);
    Ok(())
}

/// Load activity 101 info for activity 13108
pub async fn load_activity101_13108(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/static/activity101/activity101_infos_13108.json");
    load_activity101_from_json(tx, user_id, 13108, json_str).await
}

/// Load activity 101 info for activity 12722
pub async fn load_activity101_12722(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let json_str = include_str!("../../../data/static/activity101/activity101_infos_12722.json");
    load_activity101_from_json(tx, user_id, 12722, json_str).await
}

/// Helper to load activity 101 info from JSON string
async fn load_activity101_from_json(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
    activity_id: i32,
    json_str: &str,
) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();

    let data: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "load_activity101: failed to parse activity {}: {}",
                activity_id, e
            );
            return Ok(());
        }
    };

    // Load claimed days (state == 2)
    if let Some(infos) = data.get("infos").and_then(|v| v.as_array()) {
        for info in infos {
            let day_id = info.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let state = info.get("state").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            if state == 2 {
                // Already claimed
                sqlx::query(
                    "INSERT INTO user_activity101_claims (user_id, activity_id, day_id, claimed_at)
                     VALUES (?, ?, ?, ?)",
                )
                .bind(user_id)
                .bind(activity_id)
                .bind(day_id)
                .bind(now)
                .execute(&mut **tx)
                .await?;
            }
        }
    }

    // Load once bonus status
    let got_once_bonus = data
        .get("gotOnceBonus")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if got_once_bonus {
        sqlx::query(
            "INSERT INTO user_activity101_once_bonus (user_id, activity_id, claimed_at)
             VALUES (?, ?, ?)",
        )
        .bind(user_id)
        .bind(activity_id)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }

    tracing::info!("Loaded activity 101 ({}) for user {}", activity_id, user_id);
    Ok(())
}

pub async fn load_starter_bgm(tx: &mut Transaction<'_, Sqlite>, user_id: i64) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();
    let game_data = data::exceldb::get();

    let bgms: Vec<_> = game_data.bgm_switch.iter().collect();

    const DEFAULT_BGM_ID: i32 = 2204;

    for bgm in bgms {
        let is_default = bgm.id == DEFAULT_BGM_ID;
        sqlx::query(
            "INSERT INTO user_bgm (player_id, bgm_id, unlock_time, is_favorite, is_read)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(bgm.id)
        .bind(now)
        .bind(is_default)
        .bind(true)
        .execute(&mut **tx)
        .await?;
    }

    sqlx::query(
        "INSERT INTO user_bgm_state (player_id, use_bgm_id)
         VALUES (?, ?)",
    )
    .bind(user_id)
    .bind(DEFAULT_BGM_ID)
    .execute(&mut **tx)
    .await?;

    tracing::info!("Loaded starter bgm for user {}", user_id);

    Ok(())
}

pub async fn load_all_starter_data(pool: &SqlitePool, uid: i64) -> sqlx::Result<()> {
    tracing::info!("Loading all starter data for uid {uid} in a single transaction");

    let mut tx = pool.begin().await?;

    load_critter_info(&mut tx, uid).await?;
    load_player_info(&mut tx, uid).await?;
    let equip_map = load_equipment(&mut tx, uid).await?;
    // Then load heroes with equipment references
    load_hero_list(&mut tx, uid, &equip_map).await?;
    load_starter_items(&mut tx, uid).await?;
    load_starter_currencies(&mut tx, uid).await?;
    load_starter_guides(&mut tx, uid).await?;
    load_starter_user_stats(&mut tx, uid).await?;
    load_starter_hero_groups(&mut tx, uid).await?;
    load_hero_group_snapshots(&mut tx, uid).await?;
    load_dungeon_info(&mut tx, uid).await?;
    load_dungeon_infos(&mut tx, uid).await?;
    load_story_data(&mut tx, uid).await?;
    load_charge_info(&mut tx, uid).await?;
    load_block_package_info(&mut tx, uid).await?;
    load_building_info(&mut tx, uid).await?;
    load_character_interaction_info(&mut tx, uid).await?;
    load_summon_info(&mut tx, uid).await?;
    load_summon_history(&mut tx, uid).await?;
    load_achievement_info(&mut tx, uid).await?;
    load_dialog_info(&mut tx, uid).await?;
    load_starter_antiques(&mut tx, uid).await?;
    load_weekwalk_info(&mut tx, uid).await?;
    load_weekwalk_v2_info(&mut tx, uid).await?;
    load_explore_simple_info(&mut tx, uid).await?;
    load_tower_info(&mut tx, uid).await?;
    load_player_card_info(&mut tx, uid).await?;
    load_command_post_info(&mut tx, uid).await?;
    load_friend_info(&mut tx, uid).await?;
    load_simple_properties(&mut tx, uid).await?;
    load_activity101_13108(&mut tx, uid).await?;
    load_activity101_12722(&mut tx, uid).await?;
    load_starter_bgm(&mut tx, uid).await?;

    tx.commit().await?;

    tracing::info!("Finished loading all starter data for uid {uid}");
    Ok(())
}
