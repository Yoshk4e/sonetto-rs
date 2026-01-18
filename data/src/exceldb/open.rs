// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Open {
    #[serde(rename = "bindActivityId")]
    pub bind_activity_id: i32,
    pub dec: i32,
    #[serde(rename = "elementId")]
    pub element_id: i32,
    #[serde(rename = "episodeId")]
    pub episode_id: i32,
    pub id: i32,
    #[serde(rename = "isAlwaysShowBtn")]
    pub is_always_show_btn: i32,
    #[serde(rename = "isOnline")]
    pub is_online: i32,
    pub name: String,
    #[serde(rename = "playerLv")]
    pub player_lv: i32,
    #[serde(rename = "roomLevel")]
    pub room_level: i32,
    #[serde(rename = "showInEpisode")]
    pub show_in_episode: i32,
    #[serde(rename = "verifingEpisodeId")]
    pub verifing_episode_id: i32,
    #[serde(rename = "verifingHide")]
    pub verifing_hide: i32,
}
use std::collections::HashMap;

pub struct OpenTable {
    records: Vec<Open>,
    by_id: HashMap<i32, usize>,
}

impl OpenTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<Open> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&Open> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[Open] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Open> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}