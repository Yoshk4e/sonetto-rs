use data::exceldb;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

/// Load minimal critter info (one starter)
pub async fn load_critter_info(tx: &mut Transaction<'_, Sqlite>, uid: i64) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();
    let critter_uid = 10000000i64;
    sqlx::query(
        r#"
        INSERT INTO critters (
            uid, player_id, define_id, create_time,
            efficiency, patience, lucky,
            efficiency_incr_rate, patience_incr_rate, lucky_incr_rate,
            special_skin, current_mood, is_locked, finish_train, is_high_quality,
            train_hero_id, total_finish_count, name,
            created_at, updated_at
        ) VALUES (?, ?, 1, ?, 100, 100, 100, 10, 10, 10, false, 50, false, false, false, 0, 0, 'Starter Critter', ?, ?)
        "#,
    )
    .bind(critter_uid)
    .bind(uid)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Load basic player info + show heroes
pub async fn load_player_info(tx: &mut Transaction<'_, Sqlite>, uid: i64) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();
    sqlx::query(
        "INSERT INTO player_info (
            player_id, signature, birthday, portrait, show_achievement, bg,
            total_login_days, last_episode_id, last_logout_time,
            hero_rare_nn_count, hero_rare_n_count, hero_rare_r_count,
            hero_rare_sr_count, hero_rare_ssr_count,
            created_at, updated_at
        ) VALUES (?, 'Welcome to Sonetto!', '', 171603, '', 0, 1, 0, NULL, 1, 1, 1, 0, 0, ?, ?)",
    )
    .bind(uid)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Default show heroes (3 as in original)
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
        .filter(|c| c.id != 3125) // testing leveling logic
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

        let mut is_locked = is_locked;

        if equip.name_en == "Enlighten" || equip.name_en == "Gluttony" || equip.name_en == "Greed" {
            is_locked = false;
        }

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
        let mut quantity = match item.sub_type {
            13 => 0,  // potrait dupes
            48 => 10, // selector
            50 => 1,  // dev items 9999
            66 => 0,  // blocks for wilderness
            70 => 0,  // Block selector //block types are 14
            _ => 100, // Default
        };

        if item.id == 481002 {
            quantity = 5;
        }

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

        const HOUR_MS: i64 = 60 * 60 * 1000;

        let expire_time = now + insight_item.expire_hours as i64 * HOUR_MS;

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
        .bind(expire_time / 1000)
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

/// Load user stats (fixes the "User stats not found" error)
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
    .bind(0)
    .bind(true)
    .bind("用户类型7")
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn load_all_starter_data(pool: &SqlitePool, uid: i64) -> sqlx::Result<()> {
    tracing::info!(
        "Loading reduced starter data for uid {uid} (more than minimal to avoid crashes)"
    );
    let mut tx = pool.begin().await?;

    load_player_info(&mut tx, uid).await?;
    load_starter_user_stats(&mut tx, uid).await?;
    load_critter_info(&mut tx, uid).await?;
    let equip_map = load_equipment(&mut tx, uid).await?;
    load_hero_list(&mut tx, uid, &equip_map).await?;
    load_starter_items(&mut tx, uid).await?;

    // Add a few more common essentials to prevent early crashes
    // Basic guides (mark as completed)
    sqlx::query("INSERT INTO guide_progress (user_id, guide_id, step_id) VALUES (?, 1, -1)")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    // Basic summon stats
    sqlx::query(
        "INSERT INTO user_summon_stats (user_id, free_equip_summon, is_show_new_summon, new_summon_count, total_summon_count)
         VALUES (?, false, false, 0, 0)"
    )
    .bind(uid)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!("Finished loading reduced starter data for uid {uid}");
    Ok(())
}
