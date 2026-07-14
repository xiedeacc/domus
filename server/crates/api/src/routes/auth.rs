//! /auth — login, logout, admin sign-up, change password, session validate.

use crate::dto::{LoginResponseDto, UserAdminResponseDto};
use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/admin-sign-up", post(admin_sign_up))
        .route("/auth/validateToken", post(validate_token))
        .route("/auth/change-password", post(super::not_implemented))
        .route("/auth/status", axum::routing::get(super::not_implemented))
        .route("/auth/session/lock", post(super::not_implemented))
        .route("/auth/session/unlock", post(super::not_implemented))
        .route("/auth/pin-code", post(super::not_implemented))
}

#[derive(Deserialize)]
struct LoginCredentialDto {
    email: String,
    password: String,
}

async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginCredentialDto>,
) -> ApiResult<(StatusCode, HeaderMap, Json<LoginResponseDto>)> {
    let outcome = state
        .services
        .auth
        .login(&dto.email, &dto.password, ("WEB", ""))
        .await?;

    // Web clients rely on these cookies; API/mobile clients use the token.
    let mut headers = HeaderMap::new();
    for cookie in [
        format!(
            "immich_access_token={}; Path=/; HttpOnly; SameSite=Lax",
            outcome.token
        ),
        "immich_auth_type=password; Path=/; HttpOnly; SameSite=Lax".to_string(),
        "immich_is_authenticated=true; Path=/; SameSite=Lax".to_string(),
    ] {
        headers.append(SET_COOKIE, cookie.parse().unwrap());
    }

    let user = outcome.user;
    Ok((
        StatusCode::CREATED,
        headers,
        Json(LoginResponseDto {
            access_token: outcome.token,
            user_id: user.id,
            user_email: user.email.clone(),
            name: user.name.clone(),
            is_admin: user.is_admin,
            profile_image_path: user.profile_image_path.clone(),
            should_change_password: user.should_change_password,
            is_onboarded: true,
        }),
    ))
}

async fn logout(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<serde_json::Value>> {
    if let Some(session_id) = ctx.session_id {
        state.services.auth.logout(session_id).await?;
    }
    Ok(Json(serde_json::json!({
        "successful": true,
        "redirectUri": "/auth/login?autoLaunch=0"
    })))
}

#[derive(Deserialize)]
struct SignUpDto {
    email: String,
    password: String,
    name: String,
}

async fn admin_sign_up(
    State(state): State<AppState>,
    Json(dto): Json<SignUpDto>,
) -> ApiResult<(StatusCode, Json<UserAdminResponseDto>)> {
    let user = state
        .services
        .auth
        .admin_sign_up(&dto.email, &dto.password, &dto.name)
        .await?;
    Ok((StatusCode::CREATED, Json((&user).into())))
}

async fn validate_token(Auth(_ctx): Auth) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "authStatus": true }))
}
