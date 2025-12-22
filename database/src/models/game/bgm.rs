use sonettobuf;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct UserBgm {
    pub player_id: i64,
    pub bgm_id: i32,
    pub unlock_time: i64,
    pub is_favorite: bool,
    pub is_read: bool,
}

impl From<UserBgm> for sonettobuf::BgmInfo {
    fn from(bgm: UserBgm) -> Self {
        sonettobuf::BgmInfo {
            bgm_id: Some(bgm.bgm_id),
            unlock_time: Some(bgm.unlock_time as i32),
            favorite: Some(bgm.is_favorite),
            is_read: Some(bgm.is_read),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UserBgmState {
    pub player_id: i64,
    pub use_bgm_id: i32,
}
