// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub career: i32,
    pub effect: String,
    #[serde(rename = "effectHangPoint")]
    pub effect_hang_point: String,
    #[serde(rename = "energySign")]
    pub energy_sign: String,
    #[serde(rename = "heartVariantId")]
    pub heart_variant_id: i32,
    #[serde(rename = "highPriorityDes")]
    pub high_priority_des: String,
    #[serde(rename = "highPriorityName")]
    pub high_priority_name: String,
    #[serde(rename = "highPriorityNameEng")]
    pub high_priority_name_eng: String,
    #[serde(rename = "hpSign")]
    pub hp_sign: String,
    pub id: i32,
    #[serde(rename = "initial_uniqueSkill_point")]
    pub initial_unique_skill_point: i32,
    pub label: i32,
    pub level: i32,
    #[serde(rename = "levelEasy")]
    pub level_easy: i32,
    pub level_true: i32,
    #[serde(rename = "passiveSkillCount")]
    pub passive_skill_count: i32,
    #[serde(rename = "passiveSkillsEx")]
    pub passive_skills_ex: String,
    #[serde(rename = "skillTemplate")]
    pub skill_template: i32,
    #[serde(rename = "skinId")]
    pub skin_id: i32,
    pub template: i32,
    #[serde(rename = "templateEasy")]
    pub template_easy: i32,
    #[serde(rename = "uiFilterSkill")]
    pub ui_filter_skill: String,
    #[serde(rename = "uniqueSkillLevel")]
    pub unique_skill_level: i32,
}
use std::collections::HashMap;

pub struct MonsterTable {
    records: Vec<Monster>,
    by_id: HashMap<i32, usize>,
}

impl MonsterTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<Monster> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&Monster> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[Monster] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Monster> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}