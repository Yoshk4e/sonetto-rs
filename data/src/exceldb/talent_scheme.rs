// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalentScheme {
    #[serde(rename = "starMould")]
    pub star_mould: i32,
    #[serde(rename = "talenScheme")]
    pub talen_scheme: String,
    #[serde(rename = "talentId")]
    pub talent_id: i32,
    #[serde(rename = "talentMould")]
    pub talent_mould: i32,
}
pub struct TalentSchemeTable {
    records: Vec<TalentScheme>,
}

impl TalentSchemeTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<TalentScheme> = if let Some(array) = value.as_array() {
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
    pub fn all(&self) -> &[TalentScheme] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, TalentScheme> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}