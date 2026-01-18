// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterVoice {
    pub addaudio: String,
    pub audio: i32,
    pub content: String,
    pub decontent: String,
    pub deface: String,
    pub demotion: String,
    pub demouth: String,
    #[serde(rename = "displayTime")]
    pub display_time: i32,
    pub encontent: String,
    pub enface: String,
    pub enmotion: String,
    pub enmouth: String,
    pub face: String,
    pub frcontent: String,
    pub frface: String,
    pub frmotion: String,
    pub frmouth: String,
    #[serde(rename = "heroId")]
    pub hero_id: i32,
    pub jpcontent: String,
    pub jpface: String,
    pub jpmotion: String,
    pub jpmouth: String,
    pub kocontent: String,
    pub krface: String,
    pub krmotion: String,
    pub krmouth: String,
    pub motion: String,
    pub mouth: String,
    pub name: String,
    pub param: String,
    pub param2: String,
    pub show: i32,
    pub skins: String,
    #[serde(rename = "sortId")]
    pub sort_id: i32,
    #[serde(rename = "stateCondition")]
    pub state_condition: i32,
    pub thaicontent: String,
    pub thaiface: String,
    pub thaimotion: String,
    pub thaimouth: String,
    pub time: String,
    pub twcontent: String,
    pub twface: String,
    pub twmotion: String,
    pub twmouth: String,
    #[serde(rename = "type")]
    pub r#type: i32,
    #[serde(rename = "unlockCondition")]
    pub unlock_condition: String,
}
pub struct CharacterVoiceTable {
    records: Vec<CharacterVoice>,
}

impl CharacterVoiceTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<CharacterVoice> = if let Some(array) = value.as_array() {
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
    pub fn all(&self) -> &[CharacterVoice] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, CharacterVoice> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}