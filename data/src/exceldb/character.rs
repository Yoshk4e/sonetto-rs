// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub actor: String,
    pub ai: i32,
    #[serde(rename = "battleTag")]
    pub battle_tag: String,
    #[serde(rename = "birthdayBonus")]
    pub birthday_bonus: String,
    pub career: i32,
    #[serde(rename = "characterTag")]
    pub character_tag: String,
    pub desc: String,
    pub desc2: String,
    #[serde(rename = "dmgType")]
    pub dmg_type: i32,
    #[serde(rename = "duplicateItem")]
    pub duplicate_item: String,
    #[serde(rename = "duplicateItem2")]
    pub duplicate_item2: String,
    #[serde(rename = "equipRec")]
    pub equip_rec: String,
    #[serde(rename = "exSkill")]
    pub ex_skill: i32,
    #[serde(rename = "firstItem")]
    pub first_item: String,
    pub gender: i32,
    #[serde(rename = "heroType")]
    pub hero_type: i32,
    pub id: i32,
    pub initials: String,
    #[serde(rename = "isOnline")]
    pub is_online: String,
    #[serde(rename = "mvskinId")]
    pub mvskin_id: i32,
    pub name: String,
    #[serde(rename = "nameEng")]
    pub name_eng: String,
    #[serde(rename = "photoFrameBg")]
    pub photo_frame_bg: i32,
    #[serde(rename = "powerMax")]
    pub power_max: String,
    pub rank: i32,
    pub rare: i32,
    pub resistance: i32,
    #[serde(rename = "roleBirthday")]
    pub role_birthday: String,
    pub school: i32,
    pub signature: String,
    pub skill: String,
    #[serde(rename = "skinId")]
    pub skin_id: i32,
    pub stat: i32,
    pub trust: i32,
    #[serde(rename = "uniqueSkill_point")]
    pub unique_skill_point: String,
    #[serde(rename = "useDesc")]
    pub use_desc: String,
}
use std::collections::HashMap;

pub struct CharacterTable {
    records: Vec<Character>,
    by_id: HashMap<i32, usize>,
}

impl CharacterTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<Character> = if let Some(array) = value.as_array() {
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
    pub fn get(&self, id: i32) -> Option<&Character> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[Character] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Character> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}