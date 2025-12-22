use super::helpers::*;
use crate::AppState;
use crate::models::request::AccountAutoLoginReq;
use crate::models::response::AccountLoginRsp;
use axum::{extract::State, response::Json};
use common::time::ServerTime;
use database::db::user::account::TokenInfo;

pub async fn post(
    State(state): State<AppState>,
    axum::Json(req): axum::Json<AccountAutoLoginReq>,
) -> Json<AccountLoginRsp> {
    tracing::info!("Auto-login attempt - User ID: {}", req.user_id);

    // Validate token and get user
    let user = match get_user_with_token_validation(&state, req.user_id as i64, &req.token).await {
        Ok(user) => user,
        Err(e) => {
            tracing::warn!("Auto-login failed: {}", e);
            return Json(create_auth_error_response());
        }
    };

    // Generate new tokens
    let new_token = generate_token();
    let new_refresh_token = generate_token();
    let now = ServerTime::now_ms();
    let expires_in = 7 * 24 * 60 * 60 * 1000;

    // Update tokens in database
    let token_info = TokenInfo {
        token: new_token.clone(),
        refresh_token: new_refresh_token.clone(),
        expires_at: now + expires_in,
    };

    if let Err(e) = database::db::user::account::update_user_login(
        &state.game.db,
        user.user_id,
        &token_info,
        now,
    )
    .await
    {
        tracing::error!("Failed to update tokens: {}", e);
    }

    tracing::info!("Auto-login successful for user {}", req.user_id);
    Json(build_login_response(&user, new_token, new_refresh_token))
}
