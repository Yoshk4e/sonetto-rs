use super::super::utils::*;
use crate::state::battle::{effects::effect_types::EffectType, step_builder::FightStepBuilder};
use anyhow::Result;
use sonettobuf::{ActEffect, Fight, FightEntityInfo};

pub fn build_battle_start(entity: &FightEntityInfo, fight: &Fight) -> Result<Vec<ActEffect>> {
    let uid = entity.uid.unwrap_or(0);
    let team_type = entity.team_type.unwrap_or(1);
    let team_uids = get_team_uids(fight, team_type);

    let mut effects = vec![];

    for &passive_id in &entity.passive_skill {
        match passive_id {
            433011 | 434811 => {
                let effect = FightStepBuilder::new_effect()
                    .add_battle_container(
                        uid,
                        passive_id,
                        vec![
                            buff_add(uid, uid, passive_id),
                            ActEffect {
                                effect_type: Some(EffectType::TeammateInjuryCount as i32), // 210
                                target_id: Some(uid),
                                ..Default::default()
                            },
                        ],
                    )
                    .build_as_act_effect();
                effects.push(effect);
            }

            435611 => {
                let effect = FightStepBuilder::new_effect()
                    .add_battle_container(
                        uid,
                        435611,
                        vec![
                            buff_add(uid, uid, 435641),
                            attr_change(uid),
                            buff_add(uid, uid, 435611),
                            ActEffect {
                                effect_type: Some(EffectType::MasterHalo as i32), // 172
                                target_id: Some(uid),
                                ..Default::default()
                            },
                        ],
                    )
                    .build_as_act_effect();
                effects.push(effect);
            }

            435621 => {
                let mut inner = vec![];

                for target_uid in &team_uids {
                    inner.push(buff_add(*target_uid, uid, 435621));
                    inner.push(ActEffect {
                        effect_type: Some(EffectType::SlaveHalo as i32), // 173
                        target_id: Some(*target_uid),
                        ..Default::default()
                    });
                    inner.push(effect_none(*target_uid));
                }

                if !inner.is_empty() {
                    let effect = FightStepBuilder::new_effect()
                        .add_effect_container(0, 0, uid, inner)
                        .build_as_act_effect();
                    effects.push(effect);
                }
            }

            _ => {}
        }
    }

    Ok(effects)
}
