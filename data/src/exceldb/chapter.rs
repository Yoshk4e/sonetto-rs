// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    #[serde(rename = "actId")]
    pub act_id: i32,
    #[serde(rename = "ambientMusic")]
    pub ambient_music: i32,
    #[serde(rename = "canMakeTeam")]
    pub can_make_team: i32,
    #[serde(rename = "canPlayOpenMv")]
    pub can_play_open_mv: i32,
    #[serde(rename = "canReturn")]
    pub can_return: i32,
    #[serde(rename = "canUseDouble")]
    pub can_use_double: i32,
    #[serde(rename = "challengeCountLimit")]
    pub challenge_count_limit: String,
    #[serde(rename = "chapterIndex")]
    pub chapter_index: String,
    pub chapterpic: String,
    pub desc: String,
    #[serde(rename = "dramaModeToMainChapterld")]
    pub drama_mode_to_main_chapterld: i32,
    #[serde(rename = "eaActivityId")]
    pub ea_activity_id: i32,
    #[serde(rename = "elementList")]
    pub element_list: String,
    #[serde(rename = "enterAfterFreeLimit")]
    pub enter_after_free_limit: i32,
    #[serde(rename = "episodeId")]
    pub episode_id: i32,
    pub id: i32,
    #[serde(rename = "isHeroRecommend")]
    pub is_hero_recommend: i32,
    pub name: String,
    #[serde(rename = "name_En")]
    pub name_en: String,
    #[serde(rename = "navigationIcon")]
    pub navigation_icon: String,
    #[serde(rename = "openDay")]
    pub open_day: String,
    #[serde(rename = "openLevel")]
    pub open_level: i32,
    #[serde(rename = "preChapter")]
    pub pre_chapter: i32,
    #[serde(rename = "rewardPoint")]
    pub reward_point: i32,
    #[serde(rename = "rewindChapterBg")]
    pub rewind_chapter_bg: i32,
    #[serde(rename = "saveHeroGroup")]
    pub save_hero_group: bool,
    #[serde(rename = "type")]
    pub r#type: i32,
    pub year: String,
}
use std::collections::HashMap;

pub struct ChapterTable {
    records: Vec<Chapter>,
    by_id: HashMap<i32, usize>,
}

impl ChapterTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<Chapter> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&Chapter> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[Chapter] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Chapter> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}