// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthCard {
    #[serde(rename = "dailyBonus")]
    pub daily_bonus: String,
    pub days: i32,
    pub id: i32,
    #[serde(rename = "maxDaysLimit")]
    pub max_days_limit: i32,
    #[serde(rename = "onceBonus")]
    pub once_bonus: String,
    #[serde(rename = "overMaxDayBonus")]
    pub over_max_day_bonus: String,
}
use std::collections::HashMap;

pub struct MonthCardTable {
    records: Vec<MonthCard>,
    by_id: HashMap<i32, usize>,
}

impl MonthCardTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<MonthCard> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&MonthCard> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[MonthCard] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, MonthCard> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}