use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};

pub struct ServerTime;

impl ServerTime {
    /// Server reset offset (5 AM UTC reset => -5h)
    pub const RESET_OFFSET_SECONDS: i64 = -5 * 60 * 60;

    /* =========================
     * Core time helpers
     * ========================= */

    /// Current time in raw UTC milliseconds (ONLY value you store)
    #[inline]
    pub fn now_ms() -> i64 {
        Utc::now().timestamp_millis()
    }

    /// Convert UTC ms → adjusted DateTime (for comparisons only)
    #[inline]
    fn adjusted_datetime(timestamp_ms: i64) -> DateTime<Utc> {
        let utc = Utc
            .timestamp_millis_opt(timestamp_ms)
            .single()
            .expect("invalid UTC timestamp");

        utc + Duration::seconds(Self::RESET_OFFSET_SECONDS)
    }

    /* =========================
     * Server day logic
     * ========================= */

    /// Server day number (monotonic, safe for equality)
    #[inline]
    pub fn server_day(timestamp_ms: i64) -> i32 {
        Self::adjusted_datetime(timestamp_ms).num_days_from_ce()
    }

    #[inline]
    pub fn is_same_day(t1: i64, t2: i64) -> bool {
        Self::server_day(t1) == Self::server_day(t2)
    }

    #[inline]
    pub fn is_new_day(last: i64, now: i64) -> bool {
        !Self::is_same_day(last, now)
    }

    /* =========================
     * Week logic (Monday 5 AM reset)
     * ========================= */

    #[inline]
    pub fn server_week(timestamp_ms: i64) -> i32 {
        let adjusted = Self::adjusted_datetime(timestamp_ms);
        let days = adjusted.timestamp() / 86_400;
        ((days + 3) / 7) as i32 // Monday = week start
    }

    #[inline]
    pub fn is_same_week(t1: i64, t2: i64) -> bool {
        Self::server_week(t1) == Self::server_week(t2)
    }

    #[inline]
    pub fn server_weekday(timestamp_ms: i64) -> i32 {
        let dt = Self::adjusted_datetime(timestamp_ms);
        dt.weekday().num_days_from_sunday() as i32
    }

    /* =========================
     * Month logic (5 AM reset)
     * ========================= */

    #[inline]
    pub fn server_month(timestamp_ms: i64) -> i32 {
        let dt = Self::adjusted_datetime(timestamp_ms);

        // Example encoding: 2025-12 -> 202512
        dt.year() * 100 + dt.month() as i32
    }

    #[inline]
    pub fn is_same_month(t1: i64, t2: i64) -> bool {
        Self::server_month(t1) == Self::server_month(t2)
    }
}

impl ServerTime {
    /// Current server date (after reset offset)
    pub fn server_date() -> DateTime<Utc> {
        let now = Self::now_ms();
        Self::adjusted_datetime(now)
    }
}

impl ServerTime {
    pub fn now_sec_i32() -> i32 {
        (Self::now_ms() / 1000) as i32
    }
}
