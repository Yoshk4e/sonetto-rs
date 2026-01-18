use crate::state::battle::{round::RoundState, skill_executor::SkillExecutor};
use anyhow::Result;
use sonettobuf::{Fight, FightStep};
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct FightSkillMgr {
    fight: Arc<Fight>,
}

impl FightSkillMgr {
    pub fn new(fight: Arc<Fight>) -> Self {
        Self { fight }
    }

    pub fn update_fight(&mut self, fight: Arc<Fight>) {
        self.fight = fight;
    }

    pub fn execute_skill(
        &self,
        state: &RoundState,
        caster_uid: i64,
        target_uid: i64,
        skill_id: i32,
    ) -> Result<FightStep> {
        let snapshot = state.snapshot_entities_map();
        let executor = SkillExecutor::new(snapshot);
        executor.execute_skill(caster_uid, target_uid, skill_id, &state.buff_mgr)
    }
}
