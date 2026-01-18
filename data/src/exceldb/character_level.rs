// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterLevel {
    pub add_dmg: i32,
    pub atk: i32,
    pub cri: i32,
    pub cri_def: i32,
    pub cri_dmg: i32,
    pub def: i32,
    pub drop_dmg: i32,
    #[serde(rename = "heroId")]
    pub hero_id: i32,
    pub hp: i32,
    pub level: i32,
    pub mdef: i32,
    pub recri: i32,
    pub technic: i32,
}
pub struct CharacterLevelTable {
    records: Vec<CharacterLevel>,
}

impl CharacterLevelTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<CharacterLevel> = if let Some(array) = value.as_array() {
            if array.len() >= 2 && array[1].is_array() {
                serde_json::from_value(array[1].clone())?
            } else {
                serde_json::from_value(value)?
            }
        } else {
            serde_json::from_value(value)?
        };

        Ok(Self {
            records,
        })
    }

    #[inline]
    pub fn all(&self) -> &[CharacterLevel] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, CharacterLevel> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}