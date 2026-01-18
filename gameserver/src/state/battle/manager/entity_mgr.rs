use sonettobuf::{Fight, FightEntityInfo};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct EntityLocation {
    pub is_attacker: bool,
    pub index: usize,
}

#[derive(Default, Debug, Clone)]
pub struct FightEntityDataMgr {
    fight: Arc<Fight>,
    entity_cache: HashMap<i64, EntityLocation>,
}

#[allow(dead_code)]
impl FightEntityDataMgr {
    pub fn new(fight: Arc<Fight>) -> Self {
        let mut mgr = Self {
            fight: fight.clone(),
            entity_cache: HashMap::new(),
        };
        mgr.rebuild_cache();
        mgr
    }

    fn rebuild_cache(&mut self) {
        self.entity_cache.clear();

        if let Some(attacker) = self.fight.attacker.as_ref() {
            for (idx, entity) in attacker.entitys.iter().enumerate() {
                if let Some(uid) = entity.uid {
                    self.entity_cache.insert(
                        uid,
                        EntityLocation {
                            is_attacker: true,
                            index: idx,
                        },
                    );
                }
            }
        }

        if let Some(defender) = self.fight.defender.as_ref() {
            for (idx, entity) in defender.entitys.iter().enumerate() {
                if let Some(uid) = entity.uid {
                    self.entity_cache.insert(
                        uid,
                        EntityLocation {
                            is_attacker: false,
                            index: idx,
                        },
                    );
                }
            }
        }
    }

    pub fn get_by_id(&self, entity_id: i64) -> Option<&FightEntityInfo> {
        let loc = self.entity_cache.get(&entity_id)?;

        if loc.is_attacker {
            self.fight.attacker.as_ref()?.entitys.get(loc.index)
        } else {
            self.fight.defender.as_ref()?.entitys.get(loc.index)
        }
    }

    pub fn get_location(&self, entity_id: i64) -> Option<EntityLocation> {
        self.entity_cache.get(&entity_id).copied()
    }

    pub fn find_by_model_id(&self, model_id: i32) -> Option<&FightEntityInfo> {
        if let Some(attacker) = self.fight.attacker.as_ref() {
            for entity in &attacker.entitys {
                if entity.model_id == Some(model_id) {
                    return Some(entity);
                }
            }
        }

        if let Some(defender) = self.fight.defender.as_ref() {
            for entity in &defender.entitys {
                if entity.model_id == Some(model_id) {
                    return Some(entity);
                }
            }
        }

        None
    }

    pub fn get_team_entities(&self, team_type: i32) -> Vec<&FightEntityInfo> {
        let mut entities = Vec::new();

        if let Some(attacker) = self.fight.attacker.as_ref() {
            for entity in &attacker.entitys {
                if entity.team_type == Some(team_type) {
                    entities.push(entity);
                }
            }
        }

        if let Some(defender) = self.fight.defender.as_ref() {
            for entity in &defender.entitys {
                if entity.team_type == Some(team_type) {
                    entities.push(entity);
                }
            }
        }

        entities
    }

    pub fn update_fight(&mut self, fight: Arc<Fight>) {
        self.fight = fight;
        self.rebuild_cache();
    }
}

pub fn get_entity_mut_by_location(
    fight: &mut Fight,
    location: EntityLocation,
) -> Option<&mut FightEntityInfo> {
    if location.is_attacker {
        fight.attacker.as_mut()?.entitys.get_mut(location.index)
    } else {
        fight.defender.as_mut()?.entitys.get_mut(location.index)
    }
}
