use anyhow::Result;
use bcrypt::{DEFAULT_COST, hash, verify};
use sqlx::{Row, Sqlite, SqlitePool, Transaction, prelude::FromRow};

#[derive(Debug, Clone)]
pub struct UserAccount {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub vip_level: i32,
    pub first_join: bool,
    pub need_real_name: bool,
    pub real_name_status: bool,
    pub age: i32,
    pub is_adult: bool,
    pub need_activate: bool,
    pub cipher_mark: bool,
    pub account_tags: String,
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

/// Get user account by email
pub async fn get_user_by_email(pool: &SqlitePool, email: &str) -> Result<Option<UserAccount>> {
    let row = sqlx::query(
        "SELECT id, username, email, vip_level, first_join, need_real_name, real_name_status,
                age, is_adult, need_activate, cipher_mark, account_tags
         FROM users WHERE email = ?1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(Some(UserAccount {
            id: r.try_get("id")?,
            username: r.try_get("username")?,
            email: r.try_get("email")?,
            vip_level: r.try_get::<i64, _>("vip_level")? as i32,
            first_join: r.try_get::<i64, _>("first_join")? != 0,
            need_real_name: r.try_get::<i64, _>("need_real_name")? != 0,
            real_name_status: r.try_get::<i64, _>("real_name_status")? != 0,
            age: r.try_get::<Option<i64>, _>("age")?.unwrap_or(18) as i32,
            is_adult: r.try_get::<i64, _>("is_adult")? != 0,
            need_activate: r.try_get::<i64, _>("need_activate")? != 0,
            cipher_mark: r.try_get::<i64, _>("cipher_mark")? != 0,
            account_tags: r
                .try_get::<Option<String>, _>("account_tags")?
                .unwrap_or_default(),
        })),
        None => Ok(None),
    }
}

/// Get user account by ID
pub async fn get_user_by_id(pool: &SqlitePool, user_id: i64) -> Result<Option<UserAccount>> {
    let row = sqlx::query(
        "SELECT id, username, email, vip_level, first_join, need_real_name, real_name_status,
                age, is_adult, need_activate, cipher_mark, account_tags
         FROM users WHERE id = ?1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(Some(UserAccount {
            id: r.try_get("id")?,
            username: r.try_get("username")?,
            email: r.try_get("email")?,
            vip_level: r.try_get::<i64, _>("vip_level")? as i32,
            first_join: r.try_get::<i64, _>("first_join")? != 0,
            need_real_name: r.try_get::<i64, _>("need_real_name")? != 0,
            real_name_status: r.try_get::<i64, _>("real_name_status")? != 0,
            age: r.try_get::<Option<i64>, _>("age")?.unwrap_or(18) as i32,
            is_adult: r.try_get::<i64, _>("is_adult")? != 0,
            need_activate: r.try_get::<i64, _>("need_activate")? != 0,
            cipher_mark: r.try_get::<i64, _>("cipher_mark")? != 0,
            account_tags: r
                .try_get::<Option<String>, _>("account_tags")?
                .unwrap_or_default(),
        })),
        None => Ok(None),
    }
}

/// Verify user password
pub async fn verify_user_password(pool: &SqlitePool, email: &str, password: &str) -> Result<bool> {
    let row = sqlx::query("SELECT password_hash FROM users WHERE email = ?1")
        .bind(email)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(r) => {
            let password_hash: String = r.try_get("password_hash")?;
            Ok(verify(password, &password_hash)?)
        }
        None => Ok(false),
    }
}

/// Create a new user account
pub async fn create_user(
    pool: &SqlitePool,
    user_id: i64,
    email: &str,
    password: &str,
    token_info: &TokenInfo,
    now: i64,
) -> Result<UserAccount> {
    // Hash password
    let password_hash = hash(password, DEFAULT_COST)?;
    // Generate initial username from email (user can change later)
    let username = email.split('@').next().unwrap_or(email).to_string();

    sqlx::query(
        "INSERT INTO users (
            id, username, email, password_hash, account_type, registration_account_type,
            token, refresh_token, token_expires_at,
            vip_level, level, exp,
            need_real_name, real_name_status, age, is_adult,
            need_activate, cipher_mark, first_join, account_tags,
            created_at, updated_at, last_login_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)"
    )
    .bind(user_id)
    .bind(&username)
    .bind(email)
    .bind(&password_hash)
    .bind(10) // AccountType::Email
    .bind(1)
    .bind(&token_info.token)
    .bind(&token_info.refresh_token)
    .bind(token_info.expires_at)
    .bind(0) // vip_level
    .bind(80) // level
    .bind(0) // exp
    .bind(false) // need_real_name
    .bind(true)  // real_name_status
    .bind(18)    // age
    .bind(true)  // is_adult
    .bind(false) // need_activate
    .bind(true)  // cipher_mark
    .bind(false) // first_join
    .bind("") // account_tags
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // Create player_state for new user with new timestamp-based schema
    use common::time::ServerTime;

    let now_ms = now;
    let server_day = ServerTime::server_day(now_ms);

    sqlx::query(
        "INSERT INTO player_state (
            player_id,
            initial_login_complete,
            last_login_timestamp,
            created_at,
            updated_at,

            last_state_push_sent_timestamp,
            last_activity_push_sent_timestamp,
            last_currency_push_sent_timestamp,

            last_daily_reward_time,
            last_daily_reset_time,

            month_card_claimed,
            last_month_card_claim_timestamp,

            last_sign_in_day,
            last_sign_in_time,

            vip_level,
            last_energy_refill_time,
            last_weekly_reset_time,
            last_monthly_reset_time
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8,
            ?9, ?10,
            ?11, ?12,
            ?13, ?14,
            ?15, ?16, ?17, ?18
        )",
    )
    .bind(user_id)
    .bind(false) // initial_login_complete
    .bind(now) // last_login_timestamp
    .bind(now) // created_at
    .bind(now) // updated_at
    .bind(None::<i64>) // last_state_push_sent_timestamp
    .bind(None::<i64>) // last_activity_push_sent_timestamp
    .bind(None::<i64>) // last_currency_push_sent_timestamp
    .bind(None::<i64>) // last_daily_reward_time
    .bind(None::<i64>) // last_daily_reset_time (IMPORTANT)
    .bind(false) // month_card_claimed
    .bind(None::<i64>) // last_month_card_claim_timestamp
    .bind(server_day) // last_sign_in_day (server_day, not YYYYMMDD)
    .bind(now) // last_sign_in_time
    .bind(0) // vip_level
    .bind(None::<i64>) // last_energy_refill_time
    .bind(None::<i64>) // last_weekly_reset_time
    .bind(None::<i64>) // last_monthly_reset_time
    .execute(pool)
    .await?;

    // Load all starter data (critters, achievements, items, etc.)
    tracing::info!("Loading starter data for new user {}", user_id);
    if let Err(e) = crate::db::starter_data::load_all_starter_data(pool, user_id).await {
        tracing::error!("Failed to load starter data for user {}: {}", user_id, e);
        // Don't fail user creation, but log the error
    }

    Ok(UserAccount {
        id: user_id,
        username,
        email: email.to_string(),
        vip_level: 0,
        first_join: false,
        need_real_name: false,
        real_name_status: true,
        age: 18,
        is_adult: true,
        need_activate: false,
        cipher_mark: true,
        account_tags: String::new(),
    })
}

/// Update user tokens and last login time
pub async fn update_user_login(
    pool: &SqlitePool,
    user_id: i64,
    token_info: &TokenInfo,
    now: i64,
) -> Result<()> {
    sqlx::query(
        "UPDATE users SET
            token = ?1,
            refresh_token = ?2,
            token_expires_at = ?3,
            last_login_at = ?4,
            updated_at = ?5
         WHERE id = ?6",
    )
    .bind(&token_info.token)
    .bind(&token_info.refresh_token)
    .bind(token_info.expires_at)
    .bind(now)
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await?;

    // Update player_state last login
    sqlx::query(
        "UPDATE player_state SET
            last_login_timestamp = ?1,
            updated_at = ?2
         WHERE player_id = ?3",
    )
    .bind(now)
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await
    .ok();

    Ok(())
}

/// Handle user login with password verification - creates or updates user
pub async fn handle_user_login(
    pool: &SqlitePool,
    email: &str,
    password: &str,
    token_info: TokenInfo,
    now: i64,
) -> Result<UserAccount> {
    match get_user_by_email(pool, email).await? {
        Some(user) => {
            // Verify password
            if !verify_user_password(pool, email, password).await? {
                return Err(anyhow::anyhow!("Invalid password"));
            }

            // Update tokens
            update_user_login(pool, user.id, &token_info, now).await?;
            Ok(user)
        }
        None => {
            // Create new user with hashed password
            let user_id = generate_user_id(email);
            create_user(pool, user_id, email, password, &token_info, now).await
        }
    }
}

#[derive(FromRow)]
pub struct UserToken {
    pub token: String,
}

pub async fn get_user_token(pool: &SqlitePool, user_id: i64) -> Result<UserToken> {
    let token = sqlx::query_as::<_, UserToken>(
        "SELECT token
         FROM users
         WHERE id = ?1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    Ok(token)
}

pub async fn rename_user_and_update_guide(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    guide_id: i32,
    step_id: i32,
) -> Result<()> {
    let mut tx: Transaction<'_, Sqlite> = pool.begin().await?;

    // Update username
    sqlx::query(
        r#"
        UPDATE users
        SET username = ?
        WHERE id = ?
        "#,
    )
    .bind(username)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Upsert guide progress
    sqlx::query(
        r#"
        INSERT INTO guide_progress (user_id, guide_id, step_id)
        VALUES (?, ?, ?)
        ON CONFLICT(user_id, guide_id)
        DO UPDATE SET step_id = excluded.step_id
        "#,
    )
    .bind(user_id)
    .bind(guide_id)
    .bind(step_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Generate a deterministic user ID from email
fn generate_user_id(email: &str) -> i64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    email.to_lowercase().hash(&mut hasher);
    let hash = hasher.finish();

    // Ensure it's a reasonable range (e.g., 1000000 - 9999999)
    (1000000 + (hash % 9000000)) as i64
}
