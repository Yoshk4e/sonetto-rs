use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};

pub struct ServerTime;

const DAY_MS: i64 = 86_400_000;
const RESET_OFFSET_MS: i64 = 5 * 60 * 60 * 1000;
const RESET_OFFSET_SEC: i64 = 5 * 60 * 60;

impl ServerTime {
    #[inline]
    pub fn now_ms() -> i64 {
        Utc::now().timestamp_millis()
    }

    #[inline]
    pub fn adjusted_datetime(timestamp_ms: i64) -> DateTime<Utc> {
        let utc = Utc
            .timestamp_millis_opt(timestamp_ms)
            .single()
            .expect("invalid UTC timestamp");

        utc - Duration::seconds(RESET_OFFSET_SEC)
    }

    #[inline]
    pub fn server_day(now_ms: i64) -> i64 {
        (now_ms - RESET_OFFSET_MS) / DAY_MS
    }

    #[inline]
    pub fn day_of_month(timestamp_ms: i64) -> u32 {
        Self::adjusted_datetime(timestamp_ms).day()
    }

    #[inline]
    pub fn is_same_day(t1: i64, t2: i64) -> bool {
        Self::server_day(t1) == Self::server_day(t2)
    }

    #[inline]
    pub fn is_new_day(last: i64, now: i64) -> bool {
        !Self::is_same_day(last, now)
    }

    #[inline]
    pub fn server_week(timestamp_ms: i64) -> i32 {
        let adjusted = Self::adjusted_datetime(timestamp_ms);
        let days = adjusted.timestamp() / 86_400;
        ((days + 3) / 7) as i32
    }

    #[inline]
    pub fn is_same_week(t1: i64, t2: i64) -> bool {
        Self::server_week(t1) == Self::server_week(t2)
    }

    #[inline]
    pub fn server_weekday(timestamp_ms: i64) -> i32 {
        Self::adjusted_datetime(timestamp_ms)
            .weekday()
            .num_days_from_sunday() as i32
    }

    #[inline]
    pub fn server_month(timestamp_ms: i64) -> i32 {
        let dt = Self::adjusted_datetime(timestamp_ms);
        dt.year() * 100 + dt.month() as i32
    }

    #[inline]
    pub fn is_same_month(t1: i64, t2: i64) -> bool {
        Self::server_month(t1) == Self::server_month(t2)
    }

    pub fn server_date() -> DateTime<Utc> {
        Self::adjusted_datetime(Self::now_ms())
    }

    #[inline]
    pub fn now_sec_i32() -> i32 {
        (Self::now_ms() / 1000) as i32
    }
}
