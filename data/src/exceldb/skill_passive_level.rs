// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPassiveLevel {
    #[serde(rename = "heroId")]
    pub hero_id: i32,
    #[serde(rename = "skillGroup")]
    pub skill_group: i32,
    #[serde(rename = "skillLevel")]
    pub skill_level: i32,
    #[serde(rename = "skillPassive")]
    pub skill_passive: i32,
    #[serde(rename = "uiFilterSkill")]
    pub ui_filter_skill: String,
}
pub struct SkillPassiveLevelTable {
    records: Vec<SkillPassiveLevel>,
}

impl SkillPassiveLevelTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<SkillPassiveLevel> = if let Some(array) = value.as_array() {
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
    pub fn all(&self) -> &[SkillPassiveLevel] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, SkillPassiveLevel> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}