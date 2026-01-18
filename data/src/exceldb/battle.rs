// Auto-generated from JSON data
// Do not edit manually

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Battle {
    #[serde(rename = "actionRule")]
    pub action_rule: i32,
    #[serde(rename = "additionRule")]
    pub addition_rule: String,
    #[serde(rename = "advancedCondition")]
    pub advanced_condition: String,
    #[serde(rename = "aiLink")]
    pub ai_link: i32,
    pub aid: String,
    pub balance: String,
    #[serde(rename = "battleEffectiveness")]
    pub battle_effectiveness: i32,
    pub bgmevent: i32,
    #[serde(rename = "bossHpType")]
    pub boss_hp_type: String,
    #[serde(rename = "clothSkill")]
    pub cloth_skill: String,
    #[serde(rename = "dialogParam")]
    pub dialog_param: String,
    #[serde(rename = "equipEffectiveness")]
    pub equip_effectiveness: f32,
    #[serde(rename = "fightDec")]
    pub fight_dec: String,
    #[serde(rename = "fightTitle")]
    pub fight_title: String,
    #[serde(rename = "focusMonsterId")]
    pub focus_monster_id: String,
    #[serde(rename = "heroEffectiveness")]
    pub hero_effectiveness: f32,
    #[serde(rename = "hiddenRule")]
    pub hidden_rule: String,
    pub id: i32,
    #[serde(rename = "maxRound")]
    pub max_round: i32,
    #[serde(rename = "monsterGroupIds")]
    pub monster_group_ids: String,
    #[serde(rename = "monsterMax")]
    pub monster_max: i32,
    #[serde(rename = "myStance")]
    pub my_stance: String,
    #[serde(rename = "noAutoFight")]
    pub no_auto_fight: i32,
    #[serde(rename = "noClothSkill")]
    pub no_cloth_skill: i32,
    #[serde(rename = "onlyTrial")]
    pub only_trial: i32,
    #[serde(rename = "playerMax")]
    pub player_max: i32,
    #[serde(rename = "previewSkinId")]
    pub preview_skin_id: i32,
    #[serde(rename = "restrictReason")]
    pub restrict_reason: String,
    #[serde(rename = "restrictRoles")]
    pub restrict_roles: String,
    #[serde(rename = "roleNum")]
    pub role_num: i32,
    #[serde(rename = "sceneIds")]
    pub scene_ids: String,
    #[serde(rename = "talentEffectiveness")]
    pub talent_effectiveness: f32,
    #[serde(rename = "trialEquips")]
    pub trial_equips: String,
    #[serde(rename = "trialHeros")]
    pub trial_heros: String,
    #[serde(rename = "trialLimit")]
    pub trial_limit: i32,
    #[serde(rename = "trialMainAct104EuqipId")]
    pub trial_main_act104_euqip_id: i32,
    #[serde(rename = "useTemp")]
    pub use_temp: i32,
    #[serde(rename = "winCondition")]
    pub win_condition: String,
}
use std::collections::HashMap;

pub struct BattleTable {
    records: Vec<Battle>,
    by_id: HashMap<i32, usize>,
    by_group: HashMap<String, Vec<usize>>,
}

impl BattleTable {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;

        let records: Vec<Battle> = if let Some(array) = value.as_array() {
            if array.len() >= 2 && array[1].is_array() {
                serde_json::from_value(array[1].clone())?
            } else {
                serde_json::from_value(value)?
            }
        } else {
            serde_json::from_value(value)?
        };

        let mut by_id = HashMap::with_capacity(records.len());
        let mut by_group: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, record) in records.iter().enumerate() {
            by_id.insert(record.id, idx);
            by_group.entry(record.monster_group_ids.clone()).or_default().push(idx);
        }

        Ok(Self {
            records,
            by_id,
            by_group,
        })
    }

    #[inline]
    pub fn get(&self, id: i32) -> Option<&Battle> {
        self.by_id.get(&id).map(|&i| &self.records[i])
    }

    pub fn by_group(&self, group_id: String) -> impl Iterator<Item = &'_ Battle> + '_ {
        self.by_group
            .get(&group_id)
            .into_iter()
            .flat_map(|idxs| idxs.iter())
            .map(|&i| &self.records[i])
    }

    #[inline]
    pub fn all(&self) -> &[Battle] {
        &self.records
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Battle> {
        self.records.iter()
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}