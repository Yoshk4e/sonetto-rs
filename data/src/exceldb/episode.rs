// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    #[serde(rename = "advancedBonus")]
    pub advanced_bonus: i32,
    #[serde(rename = "afterStory")]
    pub after_story: i32,
    #[serde(rename = "autoSkipStory")]
    pub auto_skip_story: i32,
    #[serde(rename = "battleDesc")]
    pub battle_desc: String,
    #[serde(rename = "battleId")]
    pub battle_id: i32,
    #[serde(rename = "beforeStory")]
    pub before_story: i32,
    pub bgmevent: i32,
    pub bonus: i32,
    #[serde(rename = "canUseRecord")]
    pub can_use_record: i32,
    #[serde(rename = "chainEpisode")]
    pub chain_episode: i32,
    #[serde(rename = "chapterId")]
    pub chapter_id: i32,
    pub cost: String,
    #[serde(rename = "dayChangeBonus")]
    pub day_change_bonus: String,
    #[serde(rename = "dayNum")]
    pub day_num: i32,
    #[serde(rename = "decryptId")]
    pub decrypt_id: i32,
    pub desc: String,
    #[serde(rename = "displayMark")]
    pub display_mark: i32,
    #[serde(rename = "elementList")]
    pub element_list: String,
    #[serde(rename = "failCost")]
    pub fail_cost: String,
    #[serde(rename = "firstBattleId")]
    pub first_battle_id: i32,
    #[serde(rename = "firstBonus")]
    pub first_bonus: i32,
    #[serde(rename = "freeBonus")]
    pub free_bonus: i32,
    #[serde(rename = "freeDisplayList")]
    pub free_display_list: String,
    pub icon: String,
    pub id: i32,
    #[serde(rename = "lockTime")]
    pub lock_time: String,
    #[serde(rename = "mapId")]
    pub map_id: i32,
    pub name: String,
    #[serde(rename = "name_En")]
    pub name_en: String,
    pub navigationpic: i32,
    #[serde(rename = "permanentAdvancedBonus")]
    pub permanent_advanced_bonus: i32,
    #[serde(rename = "permanentBonus")]
    pub permanent_bonus: i32,
    #[serde(rename = "permanentFirstBonus")]
    pub permanent_first_bonus: i32,
    #[serde(rename = "permanentReward")]
    pub permanent_reward: i32,
    #[serde(rename = "permanentRewardDisplayList")]
    pub permanent_reward_display_list: String,
    #[serde(rename = "permanentRewardList")]
    pub permanent_reward_list: String,
    pub pic: String,
    #[serde(rename = "preEpisode")]
    pub pre_episode: i32,
    #[serde(rename = "preEpisodeId")]
    pub pre_episode_id: i32,
    #[serde(rename = "retroAdvancedBonus")]
    pub retro_advanced_bonus: i32,
    #[serde(rename = "retroBonus")]
    pub retro_bonus: i32,
    #[serde(rename = "retroFirstBonus")]
    pub retro_first_bonus: i32,
    #[serde(rename = "retroReward")]
    pub retro_reward: i32,
    #[serde(rename = "retroRewardDisplayList")]
    pub retro_reward_display_list: String,
    #[serde(rename = "retroRewardList")]
    pub retro_reward_list: String,
    pub reward: i32,
    #[serde(rename = "rewardDisplayList")]
    pub reward_display_list: String,
    #[serde(rename = "rewardList")]
    pub reward_list: String,
    #[serde(rename = "saveDayNum")]
    pub save_day_num: i32,
    pub story: String,
    pub time: String,
    #[serde(rename = "type")]
    pub r#type: i32,
    #[serde(rename = "unlockEpisode")]
    pub unlock_episode: i32,
    pub year: i32,
}
use std::collections::HashMap;

pub struct EpisodeTable {
    records: Vec<Episode>,
    by_id: HashMap<i32, usize>,
}

impl EpisodeTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<Episode> = if let Some(array) = value.as_array() {
            if array.len() >= 2 && array[1].is_array() {
                serde_json::from_value(array[1].clone())?
            } else {
                serde_json::from_value(value)?
            }
        } else {
            serde_json::from_value(value)?
        };

        let mut by_id = HashMap::with_capacity(records.len());

        for (idx, record) in records.iter().enumerate() {
            by_id.insert(record.id, idx);
        }

        Ok(Self {
            records,
            by_id,
        })
    }

    #[inline]
    pub fn get(&self, id: i32) -> Option<&Episode> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[Episode] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Episode> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}