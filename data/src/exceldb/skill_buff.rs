// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBuff {
    #[serde(rename = "animationName")]
    pub animation_name: String,
    pub audio: i32,
    pub bloommat: String,
    #[serde(rename = "delAudio")]
    pub del_audio: i32,
    #[serde(rename = "delEffect")]
    pub del_effect: String,
    #[serde(rename = "delEffectHangPoint")]
    pub del_effect_hang_point: String,
    pub desc: String,
    #[serde(rename = "duringTime")]
    pub during_time: i32,
    pub effect: String,
    #[serde(rename = "effectCount")]
    pub effect_count: i32,
    #[serde(rename = "effectHangPoint")]
    pub effect_hang_point: String,
    pub effectloop: i32,
    pub features: String,
    #[serde(rename = "iconId")]
    pub icon_id: String,
    pub id: i32,
    #[serde(rename = "isGoodBuff")]
    pub is_good_buff: i32,
    #[serde(rename = "isNoShow")]
    pub is_no_show: i32,
    pub mat: String,
    pub name: String,
    #[serde(rename = "triggerAnimationName")]
    pub trigger_animation_name: String,
    #[serde(rename = "triggerAudio")]
    pub trigger_audio: i32,
    #[serde(rename = "triggerEffect")]
    pub trigger_effect: String,
    #[serde(rename = "triggerEffectHangPoint")]
    pub trigger_effect_hang_point: String,
    #[serde(rename = "typeId")]
    pub type_id: i32,
}
use std::collections::HashMap;

pub struct SkillBuffTable {
    records: Vec<SkillBuff>,
    by_id: HashMap<i32, usize>,
}

impl SkillBuffTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<SkillBuff> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&SkillBuff> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[SkillBuff] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, SkillBuff> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}