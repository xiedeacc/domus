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
use chrono::{Duration, Utc};
use serde::Deserialize;

const AUTH_COOKIE_MAX_AGE_SECONDS: i64 = 34_560_000;

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
    let email = normalize_email(&dto.email);
    let outcome = state
        .services
        .auth
        .login(&email, &dto.password, ("WEB", ""))
        .await?;

    // Web clients rely on these cookies; API/mobile clients use the token.
    let headers = auth_headers(&outcome.token, "password");

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
    let email = normalize_email(&dto.email);
    let user = state
        .services
        .auth
        .admin_sign_up(&email, &dto.password, &dto.name)
        .await?;
    Ok((StatusCode::CREATED, Json((&user).into())))
}

async fn validate_token(Auth(_ctx): Auth) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "authStatus": true }))
}

pub(crate) fn auth_headers(token: &str, auth_type: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for cookie in auth_cookies(token, auth_type) {
        headers.append(SET_COOKIE, cookie.parse().unwrap());
    }
    headers
}

pub(crate) fn auth_cookies(token: &str, auth_type: &str) -> [String; 3] {
    let expires = (Utc::now() + Duration::seconds(AUTH_COOKIE_MAX_AGE_SECONDS))
        .to_rfc2822()
        .replace("+0000", "GMT");
    let attrs =
        format!("Max-Age={AUTH_COOKIE_MAX_AGE_SECONDS}; Path=/; Expires={expires}; SameSite=Lax");
    [
        format!("immich_access_token={token}; {attrs}; HttpOnly"),
        format!("immich_auth_type={auth_type}; {attrs}; HttpOnly"),
        format!("immich_is_authenticated=true; {attrs}"),
    ]
}

fn normalize_email(email: &str) -> String {
    email.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_email_matches_immich_auth_dto() {
        assert_eq!(normalize_email("aDmIn@IMMICH.cloud"), "admin@immich.cloud");
        assert_eq!(normalize_email("admin@local"), "admin@local");
    }

    #[test]
    fn auth_cookies_match_immich_cookie_contract() {
        let cookies = auth_cookies("access-token", "password");
        assert_eq!(cookies.len(), 3);
        assert!(cookies[0].starts_with("immich_access_token=access-token;"));
        assert!(cookies[1].starts_with("immich_auth_type=password;"));
        assert!(cookies[2].starts_with("immich_is_authenticated=true;"));
        for cookie in cookies {
            assert!(cookie.contains("Max-Age=34560000"));
            assert!(cookie.contains("Path=/"));
            assert!(cookie.contains("Expires="));
            assert!(cookie.contains("SameSite=Lax"));
        }
    }
}
