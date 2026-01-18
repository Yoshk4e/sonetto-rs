// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDestinyFacets {
    pub desc: String,
    #[serde(rename = "exchangeSkills")]
    pub exchange_skills: String,
    #[serde(rename = "facetsId")]
    pub facets_id: i32,
    pub level: i32,
    #[serde(rename = "powerAdd")]
    pub power_add: String,
}
pub struct CharacterDestinyFacetsTable {
    records: Vec<CharacterDestinyFacets>,
}

impl CharacterDestinyFacetsTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<CharacterDestinyFacets> = if let Some(array) = value.as_array() {
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
    pub fn all(&self) -> &[CharacterDestinyFacets] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, CharacterDestinyFacets> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}