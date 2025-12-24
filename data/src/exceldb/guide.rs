// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guide {
    #[serde(rename = "desc")]
    pub desc: String,
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "interruptFinish")]
    pub interrupt_finish: i32,
    #[serde(rename = "invalid")]
    pub invalid: String,
    #[serde(rename = "isOnline")]
    pub is_online: i32,
    #[serde(rename = "parallel")]
    pub parallel: i32,
    #[serde(rename = "priority")]
    pub priority: i32,
    #[serde(rename = "restart")]
    pub restart: i32,
    #[serde(rename = "trigger")]
    pub trigger: String,
}
use std::collections::HashMap;

pub struct GuideTable {
    records: Vec<Guide>,
    by_id: HashMap<i32, usize>,
}

impl GuideTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<Guide> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&Guide> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[Guide] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Guide> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}