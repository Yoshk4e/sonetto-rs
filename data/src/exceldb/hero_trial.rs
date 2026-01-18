// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroTrial {
    #[serde(rename = "act104EquipId1")]
    pub act104_equip_id1: i32,
    #[serde(rename = "act104EquipId2")]
    pub act104_equip_id2: i32,
    #[serde(rename = "attrId")]
    pub attr_id: i32,
    #[serde(rename = "equipId")]
    pub equip_id: i32,
    #[serde(rename = "equipLv")]
    pub equip_lv: i32,
    #[serde(rename = "equipRefine")]
    pub equip_refine: i32,
    #[serde(rename = "exSkillLv")]
    pub ex_skill_lv: i32,
    #[serde(rename = "facetsId")]
    pub facets_id: i32,
    pub facetslevel: i32,
    #[serde(rename = "heroId")]
    pub hero_id: i32,
    pub id: i32,
    pub level: i32,
    pub skin: i32,
    pub special: String,
    pub talent: i32,
    #[serde(rename = "trialTemplate")]
    pub trial_template: i32,
    #[serde(rename = "trialType")]
    pub trial_type: i32,
}
use std::collections::HashMap;

pub struct HeroTrialTable {
    records: Vec<HeroTrial>,
    by_id: HashMap<i32, usize>,
}

impl HeroTrialTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<HeroTrial> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&HeroTrial> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[HeroTrial] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, HeroTrial> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}