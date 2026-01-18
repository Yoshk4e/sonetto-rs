// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterSkillTemplate {
    #[serde(rename = "activeSkill")]
    pub active_skill: String,
    #[serde(rename = "baseStress")]
    pub base_stress: i32,
    pub camp: i32,
    pub career: i32,
    pub des: String,
    #[serde(rename = "dmgType")]
    pub dmg_type: i32,
    pub gender: i32,
    pub id: i32,
    pub identity: String,
    pub instance: i32,
    #[serde(rename = "maxStress")]
    pub max_stress: i32,
    pub name: String,
    #[serde(rename = "nameEng")]
    pub name_eng: String,
    #[serde(rename = "passiveSkill")]
    pub passive_skill: String,
    #[serde(rename = "powerMax")]
    pub power_max: String,
    pub property: String,
    pub race: i32,
    pub resistance: i32,
    pub template: String,
    #[serde(rename = "uniqueSkill")]
    pub unique_skill: String,
    #[serde(rename = "uniqueSkill_point")]
    pub unique_skill_point: i32,
}
use std::collections::HashMap;

pub struct MonsterSkillTemplateTable {
    records: Vec<MonsterSkillTemplate>,
    by_id: HashMap<i32, usize>,
}

impl MonsterSkillTemplateTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<MonsterSkillTemplate> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&MonsterSkillTemplate> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[MonsterSkillTemplate] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, MonsterSkillTemplate> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}