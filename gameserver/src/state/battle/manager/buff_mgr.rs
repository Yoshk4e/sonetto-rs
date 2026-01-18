use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Default, Debug, Clone)]
pub struct BuffInstance {
    pub buff_id: i32,
    pub from_uid: i64,
    pub duration: i32,
    pub stacks: i32,
}

#[derive(Default, Debug, Clone)]
pub struct BuffMgr {
    active: HashMap<i64, Vec<BuffInstance>>,
}

#[allow(dead_code)]
impl BuffMgr {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_buff(&mut self, target_uid: i64, buff_id: i32, from_uid: i64) {
        let entry = self.active.entry(target_uid).or_default();

        let game_data = data::exceldb::get();

        let duration = game_data
            .skill_buff
            .iter()
            .find(|b| b.id == buff_id)
            .map(|b| b.during_time)
            .unwrap_or(0);

        if let Some(existing) = entry.iter_mut().find(|b| b.buff_id == buff_id) {
            existing.stacks += 1;
            existing.duration = existing.duration.max(duration);
        } else {
            entry.push(BuffInstance {
                buff_id,
                from_uid,
                duration,
                stacks: 1,
            });
        }

        tracing::info!(
            "[BuffMgr] Added buff {} to {} from {} (duration={})",
            buff_id,
            target_uid,
            from_uid,
            duration
        );
    }

    pub fn get_buffs(&self, uid: i64) -> &[BuffInstance] {
        self.active.get(&uid).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn has_buff(&self, uid: i64, buff_id: i32) -> bool {
        self.active
            .get(&uid)
            .map(|b| b.iter().any(|x| x.buff_id == buff_id))
            .unwrap_or(false)
    }

    pub fn on_round_end(&mut self) {
        for buffs in self.active.values_mut() {
            for b in buffs.iter_mut() {
                b.duration -= 1;
            }
            buffs.retain(|b| b.duration > 0);
        }
    }

    pub fn clear_dead(&mut self, uid: i64) {
        self.active.remove(&uid);
    }
}
