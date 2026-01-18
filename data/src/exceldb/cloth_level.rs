// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClothLevel {
    #[serde(rename = "allLimit1")]
    pub all_limit1: i32,
    #[serde(rename = "allLimit2")]
    pub all_limit2: i32,
    #[serde(rename = "allLimit3")]
    pub all_limit3: i32,
    pub cd1: i32,
    pub cd2: i32,
    pub cd3: i32,
    pub compose: i32,
    pub death: i32,
    pub defeat: i32,
    pub desc: String,
    pub exp: i32,
    pub id: i32,
    pub initial: i32,
    pub level: i32,
    #[serde(rename = "maxPower")]
    pub max_power: i32,
    #[serde(rename = "move")]
    pub r#move: i32,
    #[serde(rename = "passiveSkills")]
    pub passive_skills: String,
    pub recover: String,
    pub skill1: i32,
    pub skill2: i32,
    pub skill3: i32,
    #[serde(rename = "use")]
    pub r#use: i32,
    #[serde(rename = "usePower1")]
    pub use_power1: Vec<i32>,
    #[serde(rename = "usePower2")]
    pub use_power2: Vec<i32>,
    #[serde(rename = "usePower3")]
    pub use_power3: Vec<serde_json::Value>,
}
use std::collections::HashMap;

pub struct ClothLevelTable {
    records: Vec<ClothLevel>,
    by_id: HashMap<i32, usize>,
}

impl ClothLevelTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<ClothLevel> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&ClothLevel> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[ClothLevel] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, ClothLevel> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}