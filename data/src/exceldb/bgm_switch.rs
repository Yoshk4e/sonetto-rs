// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgmSwitch {
    pub audio: i32,
    #[serde(rename = "audioBg")]
    pub audio_bg: String,
    #[serde(rename = "audioEvaluates")]
    pub audio_evaluates: String,
    #[serde(rename = "audioIntroduce")]
    pub audio_introduce: String,
    #[serde(rename = "audioLength")]
    pub audio_length: f32,
    #[serde(rename = "audioName")]
    pub audio_name: String,
    #[serde(rename = "audioNameEn")]
    pub audio_name_en: String,
    #[serde(rename = "audioType")]
    pub audio_type: i32,
    pub audioicon: String,
    #[serde(rename = "defaultUnlock")]
    pub default_unlock: i32,
    pub id: i32,
    #[serde(rename = "isNonLoop")]
    pub is_non_loop: i32,
    #[serde(rename = "isReport")]
    pub is_report: i32,
    #[serde(rename = "itemId")]
    pub item_id: i32,
    pub sort: i32,
    #[serde(rename = "unlockCondition")]
    pub unlock_condition: String,
}
use std::collections::HashMap;

pub struct BgmSwitchTable {
    records: Vec<BgmSwitch>,
    by_id: HashMap<i32, usize>,
}

impl BgmSwitchTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<BgmSwitch> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&BgmSwitch> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[BgmSwitch] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, BgmSwitch> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}