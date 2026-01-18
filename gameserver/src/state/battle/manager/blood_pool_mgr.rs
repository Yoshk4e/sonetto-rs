use crate::state::battle::mechanics::bloodtithe::*;
use sonettobuf::Fight;
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct FightBloodPoolDataMgr {
    fight: Arc<Fight>,
}

impl FightBloodPoolDataMgr {
    pub fn new(fight: Arc<Fight>) -> Self {
        Self { fight }
    }

    pub fn initialize(&mut self, state: &mut BloodtitheState) {
        if state.initialized {
            return;
        }
        state.initialized = true;

        if fight_enables_bloodtithe(&self.fight) {
            let enablers = count_team_enablers(&self.fight, 1);
            state.recalc_max(1, enablers);

            if self.has_hero_3120(1) {
                state.add_initial_gain(1, 16);
            }
        }
    }

    pub fn has_hero_3120(&self, team_type: i32) -> bool {
        self.fight
            .attacker
            .as_ref()
            .map(|a| {
                a.entitys.iter().any(|e| {
                    e.team_type == Some(team_type)
                        && e.model_id == Some(3120)
                        && entity_enables_bloodtithe(e)
                })
            })
            .unwrap_or(false)
    }

    pub fn update_fight(&mut self, fight: Arc<Fight>) {
        self.fight = fight;
    }
}
