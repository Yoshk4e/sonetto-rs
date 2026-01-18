// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "activityId")]
    pub activity_id: i32,
    #[serde(rename = "boxOpen")]
    pub box_open: String,
    pub cd: i32,
    pub clienttag: i32,
    pub desc: String,
    pub effect: String,
    #[serde(rename = "expireTime")]
    pub expire_time: String,
    #[serde(rename = "headIconSign")]
    pub head_icon_sign: String,
    #[serde(rename = "highQuality")]
    pub high_quality: i32,
    pub icon: String,
    pub id: i32,
    #[serde(rename = "isDynamic")]
    pub is_dynamic: i32,
    #[serde(rename = "isShow")]
    pub is_show: i32,
    #[serde(rename = "isStackable")]
    pub is_stackable: i32,
    #[serde(rename = "isTimeShow")]
    pub is_time_show: i32,
    pub name: String,
    pub price: String,
    pub rare: i32,
    pub sources: String,
    #[serde(rename = "subType")]
    pub sub_type: i32,
    #[serde(rename = "useDesc")]
    pub use_desc: String,
}
use std::collections::HashMap;

pub struct ItemTable {
    records: Vec<Item>,
    by_id: HashMap<i32, usize>,
}

impl ItemTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<Item> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&Item> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[Item] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Item> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}