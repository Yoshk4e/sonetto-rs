// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightItem {
    pub desc: String,
    pub effect: String,
    #[serde(rename = "expireHours")]
    pub expire_hours: i32,
    #[serde(rename = "heroRank")]
    pub hero_rank: i32,
    #[serde(rename = "heroRares")]
    pub hero_rares: String,
    pub icon: String,
    pub id: i32,
    pub name: String,
    pub rare: i32,
    pub sources: String,
    #[serde(rename = "useDesc")]
    pub use_desc: String,
    #[serde(rename = "useTitle")]
    pub use_title: String,
}
use std::collections::HashMap;

pub struct InsightItemTable {
    records: Vec<InsightItem>,
    by_id: HashMap<i32, usize>,
}

impl InsightItemTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<InsightItem> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&InsightItem> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[InsightItem] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, InsightItem> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}