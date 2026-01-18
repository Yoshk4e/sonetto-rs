// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipSkill {
    pub absorb: i32,
    #[serde(rename = "addDmg")]
    pub add_dmg: i32,
    pub attack: i32,
    #[serde(rename = "baseDesc")]
    pub base_desc: String,
    #[serde(rename = "baseDesc2")]
    pub base_desc2: String,
    pub career: String,
    pub clutch: i32,
    pub condition: String,
    pub cri: i32,
    #[serde(rename = "criDef")]
    pub cri_def: i32,
    #[serde(rename = "criDmg")]
    pub cri_dmg: i32,
    #[serde(rename = "defenseIgnore")]
    pub defense_ignore: i32,
    #[serde(rename = "dropDmg")]
    pub drop_dmg: i32,
    pub heal: i32,
    pub hp: i32,
    pub id: i32,
    #[serde(rename = "normalSkillRate")]
    pub normal_skill_rate: i32,
    pub recri: i32,
    pub revive: i32,
    pub skill: i32,
    pub skill2: i32,
    #[serde(rename = "skillHide")]
    pub skill_hide: String,
    #[serde(rename = "skillLv")]
    pub skill_lv: i32,
}
use std::collections::HashMap;

pub struct EquipSkillTable {
    records: Vec<EquipSkill>,
    by_id: HashMap<i32, usize>,
}

impl EquipSkillTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<EquipSkill> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&EquipSkill> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[EquipSkill] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, EquipSkill> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}