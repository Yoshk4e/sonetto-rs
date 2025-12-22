use crate::AppState;
use crate::models::response::{
    AccountLoginRsp, AccountLoginRspData, AccountType, PageDatum, PoolName, RealNameInfo,
    SummonQueryRsp, SummonQueryRspData,
};
use anyhow::Result;
use common::time::ServerTime;
use rand::Rng;
use sqlx::Row;
use sqlx::prelude::FromRow;

/// Shared user data struct from database
#[allow(dead_code)]
pub struct UserData {
    pub user_id: i64,
    pub username: String,
    pub email: String,
    pub first_join: bool,
    pub real_name_status: bool,
    pub age: i32,
    pub is_adult: bool,
    pub account_tags: String,
    pub need_activate: bool,
    pub cipher_mark: bool,
    pub token: String,
    pub refresh_token: String,
    pub token_expires_at: Option<i64>,
    pub created_at: Option<i64>,
    pub last_login_at: Option<i64>,
}

/// Fetch user from database with token validation
pub async fn get_user_with_token_validation(
    state: &AppState,
    user_id: i64,
    token: &str,
) -> Result<UserData> {
    let row = sqlx::query(
        "SELECT username, email, first_join, real_name_status, age, is_adult, account_tags,
                need_activate, cipher_mark, token, refresh_token, token_expires_at,
                created_at, last_login_at
         FROM users WHERE id = ?1",
    )
    .bind(user_id)
    .fetch_optional(&state.game.db)
    .await?
    .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    // Validate token
    let stored_token: String = row.try_get("token")?;
    if stored_token != token {
        return Err(anyhow::anyhow!("Invalid token"));
    }

    Ok(UserData {
        user_id,
        username: row.try_get("username")?,
        email: row.try_get("email")?,
        first_join: row.try_get::<i64, _>("first_join")? != 0,
        real_name_status: row.try_get::<i64, _>("real_name_status")? != 0,
        age: row.try_get::<Option<i64>, _>("age")?.unwrap_or(18) as i32,
        is_adult: row.try_get::<i64, _>("is_adult")? != 0,
        account_tags: row
            .try_get::<Option<String>, _>("account_tags")?
            .unwrap_or_default(),
        need_activate: row.try_get::<i64, _>("need_activate")? != 0,
        cipher_mark: row.try_get::<i64, _>("cipher_mark")? != 0,
        token: row.try_get("token")?,
        refresh_token: row.try_get("refresh_token")?,
        token_expires_at: row.try_get("token_expires_at").ok(),
        created_at: row.try_get("created_at").ok(),
        last_login_at: row.try_get("last_login_at").ok(),
    })
}

/// Fetch user from database by ID only
pub async fn get_user_by_id(state: &AppState, user_id: i64) -> Result<UserData> {
    let row = sqlx::query(
        "SELECT username, email, first_join, real_name_status, age, is_adult, account_tags,
                need_activate, cipher_mark, token, refresh_token, token_expires_at,
                created_at, last_login_at
         FROM users WHERE id = ?1",
    )
    .bind(user_id)
    .fetch_optional(&state.game.db)
    .await?
    .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    Ok(UserData {
        user_id,
        username: row.try_get("username")?,
        email: row.try_get("email")?,
        first_join: row.try_get::<i64, _>("first_join")? != 0,
        real_name_status: row.try_get::<i64, _>("real_name_status")? != 0,
        age: row.try_get::<Option<i64>, _>("age")?.unwrap_or(18) as i32,
        is_adult: row.try_get::<i64, _>("is_adult")? != 0,
        account_tags: row
            .try_get::<Option<String>, _>("account_tags")?
            .unwrap_or_default(),
        need_activate: row.try_get::<i64, _>("need_activate")? != 0,
        cipher_mark: row.try_get::<i64, _>("cipher_mark")? != 0,
        token: row.try_get("token")?,
        refresh_token: row.try_get("refresh_token")?,
        token_expires_at: row.try_get("token_expires_at").ok(),
        created_at: row.try_get("created_at").ok(),
        last_login_at: row.try_get("last_login_at").ok(),
    })
}

/// Generate a random token
pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
        + "200"
}

/// Generate a random session ID
pub fn generate_session_id() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Mask email for display (e.g., "steve@gmail.com" -> "ste****@gmail.com")
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        let prefix_len = local.chars().count().min(3);
        format!(
            "{}****{}",
            local.chars().take(prefix_len).collect::<String>(),
            domain
        )
    } else {
        email.to_string()
    }
}

/// Calculate remaining token expiry time
pub fn calculate_expires_in(token_expires_at: Option<i64>) -> i64 {
    let now = ServerTime::now_ms() as i64;
    token_expires_at
        .map(|exp| ((exp - now) / 1000).max(0))
        .unwrap_or(604800)
}

/// Format timestamp to readable string
pub fn format_timestamp(timestamp: Option<i64>) -> String {
    timestamp
        .and_then(|ts| {
            chrono::DateTime::from_timestamp((ts / 1000) as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        })
        .unwrap_or_else(|| "2025-04-12 19:40:00".to_string())
}

/// Create a standard error response for authentication failures
pub fn create_auth_error_response() -> AccountLoginRsp {
    AccountLoginRsp {
        code: 401,
        msg: "Invalid credentials or token".to_string(),
        data: AccountLoginRspData {
            token: String::new(),
            expires_in: 0,
            refresh_token: String::new(),
            user_id: 0,
            account_type: AccountType::Email,
            registration_account_type: 0,
            account: String::new(),
            real_name_info: RealNameInfo {
                need_real_name: false,
                real_name_status: false,
                age: 0,
                adult: false,
            },
            need_activate: false,
            cipher_mark: false,
            first_join: false,
            account_tags: String::new(),
        },
    }
}

/// Build standard login response from user data
pub fn build_login_response(
    user: &UserData,
    new_token: String,
    new_refresh_token: String,
) -> AccountLoginRsp {
    let expires_in = calculate_expires_in(user.token_expires_at);

    AccountLoginRsp {
        code: 200,
        msg: "success".to_string(),
        data: AccountLoginRspData {
            token: new_token,
            expires_in,
            refresh_token: new_refresh_token,
            user_id: user.user_id as u64,
            account_type: AccountType::Email,
            registration_account_type: 1,
            account: mask_email(&user.email),
            real_name_info: RealNameInfo {
                need_real_name: !user.real_name_status,
                real_name_status: user.real_name_status,
                age: user.age as u8,
                adult: user.is_adult,
            },
            need_activate: user.need_activate,
            cipher_mark: user.cipher_mark,
            first_join: user.first_join,
            account_tags: user.account_tags.clone(),
        },
    }
}

/// Fetch summon history with token validation
pub async fn get_summons(
    state: &AppState,
    user_id: i64,
    token: &str,
) -> anyhow::Result<SummonQueryRsp> {
    // --- Validate user + token (same pattern as user loader)
    let row = sqlx::query(
        "SELECT token
         FROM users
         WHERE id = ?1",
    )
    .bind(user_id)
    .fetch_optional(&state.game.db)
    .await?
    .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    let stored_token: String = row.try_get("token")?;
    if stored_token != token {
        return Err(anyhow::anyhow!("Invalid token"));
    }

    #[allow(dead_code)]
    #[derive(FromRow)]
    struct SummonHistory {
        id: i64,
        summon_type: i32,
        summon_time: i64,
        pool_id: i64,
        pool_type: i32,
        pool_name: String,
    }

    // --- Load summon history
    let history_rows = sqlx::query_as::<_, SummonHistory>(
        "
        SELECT
            id,
            summon_type,
            summon_time,
            pool_id,
            pool_type,
            pool_name
        FROM user_summon_history
        WHERE user_id = ?
        ORDER BY summon_time DESC
        ",
    )
    .bind(user_id)
    .fetch_all(&state.game.db)
    .await?;

    let mut page_data = Vec::with_capacity(history_rows.len());

    #[derive(FromRow)]
    struct SummonHistoryItem {
        gain_id: i64,
    }

    for history in history_rows {
        let item_rows = sqlx::query_as::<_, SummonHistoryItem>(
            "
            SELECT gain_id
            FROM user_summon_history_items
            WHERE history_id = ?
            ORDER BY result_index ASC
            ",
        )
        .bind(history.id)
        .fetch_all(&state.game.db)
        .await?;

        let gain_ids = item_rows
            .into_iter()
            .map(|r| r.gain_id as i64)
            .collect::<Vec<_>>();

        page_data.push(PageDatum {
            summon_type: history.summon_type.to_string(),
            lucky_bag_ids: Vec::new(),
            create_time: format_timestamp(Some(history.summon_time)),
            pool_id: history.pool_id as i64,
            gain_ids,
            pool_type: history.pool_type as i64,
            pool_name: PoolName::from_db(&history.pool_name),
        });
    }

    Ok(SummonQueryRsp {
        code: 200,
        msg: "成功".to_string(),
        data: SummonQueryRspData { page_data },
    })
}
