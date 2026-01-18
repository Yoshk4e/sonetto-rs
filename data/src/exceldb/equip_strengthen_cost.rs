// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipStrengthenCost {
    #[serde(rename = "attributeRate")]
    pub attribute_rate: i32,
    pub exp: i32,
    pub level: i32,
    pub rare: i32,
    #[serde(rename = "scoreCost")]
    pub score_cost: i32,
}
pub struct EquipStrengthenCostTable {
    records: Vec<EquipStrengthenCost>,
}

impl EquipStrengthenCostTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<EquipStrengthenCost> = if let Some(array) = value.as_array() {
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
    pub fn all(&self) -> &[EquipStrengthenCost] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, EquipStrengthenCost> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}