use anyhow::Result;
use rand::SeedableRng;
use rand::rngs::StdRng;
use sonettobuf::{BeginRoundOper, CardInfo, FightRound};

use crate::state::battle::manager::fight_data_mgr::FightDataMgr;

pub struct BattleSimulator {
    rng: StdRng,
    data: FightDataMgr,
}

impl BattleSimulator {
    pub fn new(data: FightDataMgr) -> Self {
        let seed = chrono::Utc::now().timestamp_millis() as u64;

        let fight = data.get_fight_snapshot();
        tracing::info!(
            "Initialized battle with {} player entities, {} enemy entities",
            fight
                .attacker
                .as_ref()
                .map(|a| a.entitys.len())
                .unwrap_or(0),
            fight
                .defender
                .as_ref()
                .map(|d| d.entitys.len())
                .unwrap_or(0),
        );

        Self {
            rng: StdRng::seed_from_u64(seed),
            data,
        }
    }

    pub async fn process_round(
        &mut self,
        operations: Vec<BeginRoundOper>,
        current_deck: Vec<CardInfo>,
        ai_deck: Vec<CardInfo>,
    ) -> Result<FightRound> {
        let round = {
            let (round_mgr, card_mgr, calc, fight, bloodtithe, buff_mgr) =
                self.data.split_all_mut();

            round_mgr
                .process_round(
                    &mut self.rng,
                    card_mgr,
                    calc,
                    fight,
                    bloodtithe,
                    operations,
                    current_deck,
                    ai_deck,
                    buff_mgr,
                )
                .await?
        };

        self.data.update_managers();
        Ok(round)
    }
}
