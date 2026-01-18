// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    #[serde(rename = "activeTargetFrameEvent")]
    pub active_target_frame_event: String,
    #[serde(rename = "battleTag")]
    pub battle_tag: String,
    #[serde(rename = "bloomParams")]
    pub bloom_params: String,
    pub desc_art: String,
    pub eff_desc: String,
    #[serde(rename = "heroId")]
    pub hero_id: i32,
    pub icon: i32,
    pub id: i32,
    pub name: String,
    #[serde(rename = "notDoAction")]
    pub not_do_action: i32,
    #[serde(rename = "preFxId")]
    pub pre_fx_id: i32,
    #[serde(rename = "showInBattle")]
    pub show_in_battle: i32,
    #[serde(rename = "skillEffect")]
    pub skill_effect: i32,
    #[serde(rename = "skillRank")]
    pub skill_rank: i32,
    pub timeline: String,
}
use std::collections::HashMap;

pub struct SkillTable {
    records: Vec<Skill>,
    by_id: HashMap<i32, usize>,
}

impl SkillTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<Skill> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&Skill> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[Skill] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Skill> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}