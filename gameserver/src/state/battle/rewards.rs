use data::exceldb;

#[derive(Debug, Clone)]
pub struct DungeonRewards {
    pub normal_bonus: Vec<(u32, u32, i32)>, // (type, id, amount)
    pub first_bonus: Vec<(u32, u32, i32)>,
    pub free_bonus: Vec<(u32, u32, i32)>,
}

/// Generate dungeon rewards from episode data
pub fn generate_dungeon_rewards(
    episode_id: i32,
    is_first_clear: bool,
    multiplication: i32, // From StartDungeonRequest
) -> DungeonRewards {
    let game_data = exceldb::get();

    let episode = game_data.episode.iter().find(|e| e.id == episode_id);

    let Some(episode) = episode else {
        return DungeonRewards {
            normal_bonus: vec![],
            first_bonus: vec![],
            free_bonus: vec![],
        };
    };

    let normal_bonus = if episode.bonus != 0 {
        parse_bonus_rewards(episode.bonus, multiplication)
    } else {
        vec![]
    };

    // Parse first clear bonus (only on first completion)
    let first_bonus = if is_first_clear && episode.first_bonus != 0 {
        parse_bonus_rewards(episode.first_bonus, multiplication)
    } else {
        vec![]
    };

    // Parse free bonus
    let free_bonus = if episode.free_bonus != 0 {
        parse_bonus_rewards(episode.free_bonus, multiplication)
    } else {
        vec![]
    };

    DungeonRewards {
        normal_bonus,
        first_bonus,
        free_bonus,
    }
}

/// Parse bonus table entry and extract rewards
fn parse_bonus_rewards(bonus_id: i32, multiplication: i32) -> Vec<(u32, u32, i32)> {
    let game_data = exceldb::get();
    // Get bonus data
    let bonus = game_data.bonus.iter().find(|b| b.id == bonus_id);

    let Some(bonus) = bonus else {
        return vec![];
    };

    // Parse fixBonus string: "2#21#2|9#1003#1|9#1002#3"
    let fix_bonus = bonus.fix_bonus.as_str();

    parse_reward_string(fix_bonus, multiplication)
}

/// Parse reward string format: "2#21#2|9#1003#1|9#1002#3"
/// Format: type#id#base_amount separated by |
fn parse_reward_string(reward_str: &str, multiplication: i32) -> Vec<(u32, u32, i32)> {
    let mut rewards = Vec::new();

    for part in reward_str.split('|') {
        let components: Vec<&str> = part.split('#').collect();
        if components.len() >= 3
            && let (Ok(reward_type), Ok(reward_id), Ok(base_amount)) = (
                components[0].parse::<u32>(),
                components[1].parse::<u32>(),
                components[2].parse::<i32>(),
            )
        {
            // Apply multiplication (4x for example)
            let final_amount = base_amount * multiplication;
            rewards.push((reward_type, reward_id, final_amount));
        }
    }

    rewards
}
