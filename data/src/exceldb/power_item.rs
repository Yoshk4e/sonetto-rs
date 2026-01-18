// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerItem {
    pub desc: String,
    pub effect: i32,
    #[serde(rename = "expireTime")]
    pub expire_time: String,
    #[serde(rename = "expireType")]
    pub expire_type: i32,
    #[serde(rename = "highQuality")]
    pub high_quality: i32,
    pub icon: String,
    pub id: i32,
    pub name: String,
    pub rare: i32,
    pub sources: String,
    #[serde(rename = "useDesc")]
    pub use_desc: String,
}
use std::collections::HashMap;

pub struct PowerItemTable {
    records: Vec<PowerItem>,
    by_id: HashMap<i32, usize>,
}

impl PowerItemTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<PowerItem> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&PowerItem> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[PowerItem] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, PowerItem> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}