use common::time::ServerTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerState {
    pub player_id: i64,

    pub initial_login_complete: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_login_timestamp: Option<i64>,

    pub last_state_push_sent_timestamp: Option<i64>,
    pub last_activity_push_sent_timestamp: Option<i64>,

    pub last_daily_reward_time: Option<i64>,
    pub last_daily_reset_time: Option<i64>,

    pub month_card_claimed: bool,
    pub last_month_card_claim_timestamp: Option<i64>,

    pub last_sign_in_day: i64, // server_day
    pub last_sign_in_time: Option<i64>,

    pub vip_level: i32,
    pub last_energy_refill_time: Option<i64>,
    pub last_weekly_reset_time: Option<i64>,
    pub last_monthly_reset_time: Option<i64>,
}

impl PlayerState {
    pub fn new(player_id: i64, now_ms: i64) -> Self {
        let server_day = ServerTime::server_day(now_ms);

        Self {
            player_id,

            initial_login_complete: false,
            created_at: now_ms,
            updated_at: now_ms,
            last_login_timestamp: Some(now_ms),

            last_state_push_sent_timestamp: None,
            last_activity_push_sent_timestamp: None,

            last_daily_reward_time: None,
            last_daily_reset_time: None,

            month_card_claimed: false,
            last_month_card_claim_timestamp: None,

            last_sign_in_day: server_day,
            last_sign_in_time: Some(now_ms),

            vip_level: 0,
            last_energy_refill_time: None,
            last_weekly_reset_time: None,
            last_monthly_reset_time: None,
        }
    }
}

impl PlayerState {
    #[inline]
    pub fn is_new_server_day(&self, now_ms: i64) -> bool {
        match self.last_daily_reset_time {
            Some(ts) => ServerTime::server_day(ts) != ServerTime::server_day(now_ms),
            None => true,
        }
    }

    #[inline]
    pub fn is_new_reward_day(&self, now_ms: i64) -> bool {
        match self.last_daily_reward_time {
            Some(ts) => ServerTime::server_day(ts) != ServerTime::server_day(now_ms),
            None => true,
        }
    }

    #[inline]
    pub fn is_new_week(&self, now_ms: i64) -> bool {
        match self.last_weekly_reset_time {
            Some(ts) => ServerTime::server_week(ts) != ServerTime::server_week(now_ms),
            None => true,
        }
    }

    #[inline]
    pub fn is_new_month(&self, now_ms: i64) -> bool {
        match self.last_monthly_reset_time {
            Some(ts) => ServerTime::server_month(ts) != ServerTime::server_month(now_ms),
            None => true,
        }
    }
}

impl PlayerState {
    pub fn needs_state_push(&self, now_ms: i64) -> bool {
        match self.last_state_push_sent_timestamp {
            None => true,
            Some(last) => ServerTime::server_day(last) != ServerTime::server_day(now_ms),
        }
    }

    pub fn needs_activity_push(&self, now_ms: i64) -> bool {
        self.is_new_server_day(now_ms) || self.last_activity_push_sent_timestamp.is_none()
    }
}

impl PlayerState {
    pub fn mark_login_complete(&mut self, now_ms: i64) {
        self.initial_login_complete = true;
        self.updated_at = now_ms;
    }

    pub fn mark_daily_reward_claimed(&mut self, now_ms: i64) {
        self.last_daily_reward_time = Some(now_ms);
        self.updated_at = now_ms;
    }

    pub fn mark_activity_pushes_sent(&mut self, current_time: i64) {
        self.last_activity_push_sent_timestamp = Some(current_time);
        self.updated_at = current_time;
    }

    pub fn claim_month_card(&mut self, now_ms: i64) {
        self.month_card_claimed = true;
        self.last_month_card_claim_timestamp = Some(now_ms);
        self.updated_at = now_ms;
    }

    pub fn mark_state_push_sent(&mut self, now_ms: i64) {
        self.last_state_push_sent_timestamp = Some(now_ms);
        self.updated_at = now_ms;
    }

    pub fn mark_activity_push_sent(&mut self, now_ms: i64) {
        self.last_activity_push_sent_timestamp = Some(now_ms);
        self.updated_at = now_ms;
    }
}
