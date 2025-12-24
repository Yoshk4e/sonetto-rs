// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterVoice {
    #[serde(rename = "addaudio")]
    pub addaudio: String,
    #[serde(rename = "audio")]
    pub audio: i32,
    #[serde(rename = "content")]
    pub content: String,
    #[serde(rename = "decontent")]
    pub decontent: String,
    #[serde(rename = "deface")]
    pub deface: String,
    #[serde(rename = "demotion")]
    pub demotion: String,
    #[serde(rename = "demouth")]
    pub demouth: String,
    #[serde(rename = "displayTime")]
    pub display_time: i32,
    #[serde(rename = "encontent")]
    pub encontent: String,
    #[serde(rename = "enface")]
    pub enface: String,
    #[serde(rename = "enmotion")]
    pub enmotion: String,
    #[serde(rename = "enmouth")]
    pub enmouth: String,
    #[serde(rename = "face")]
    pub face: String,
    #[serde(rename = "frcontent")]
    pub frcontent: String,
    #[serde(rename = "frface")]
    pub frface: String,
    #[serde(rename = "frmotion")]
    pub frmotion: String,
    #[serde(rename = "frmouth")]
    pub frmouth: String,
    #[serde(rename = "heroId")]
    pub hero_id: i32,
    #[serde(rename = "jpcontent")]
    pub jpcontent: String,
    #[serde(rename = "jpface")]
    pub jpface: String,
    #[serde(rename = "jpmotion")]
    pub jpmotion: String,
    #[serde(rename = "jpmouth")]
    pub jpmouth: String,
    #[serde(rename = "kocontent")]
    pub kocontent: String,
    #[serde(rename = "krface")]
    pub krface: String,
    #[serde(rename = "krmotion")]
    pub krmotion: String,
    #[serde(rename = "krmouth")]
    pub krmouth: String,
    #[serde(rename = "motion")]
    pub motion: String,
    #[serde(rename = "mouth")]
    pub mouth: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "param")]
    pub param: String,
    #[serde(rename = "param2")]
    pub param2: String,
    #[serde(rename = "show")]
    pub show: i32,
    #[serde(rename = "skins")]
    pub skins: String,
    #[serde(rename = "sortId")]
    pub sort_id: i32,
    #[serde(rename = "stateCondition")]
    pub state_condition: i32,
    #[serde(rename = "thaicontent")]
    pub thaicontent: String,
    #[serde(rename = "thaiface")]
    pub thaiface: String,
    #[serde(rename = "thaimotion")]
    pub thaimotion: String,
    #[serde(rename = "thaimouth")]
    pub thaimouth: String,
    #[serde(rename = "time")]
    pub time: String,
    #[serde(rename = "twcontent")]
    pub twcontent: String,
    #[serde(rename = "twface")]
    pub twface: String,
    #[serde(rename = "twmotion")]
    pub twmotion: String,
    #[serde(rename = "twmouth")]
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