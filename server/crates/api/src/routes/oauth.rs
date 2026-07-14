//! Immich-compatible OAuth routes.

use crate::dto::{LoginResponseDto, UserAdminResponseDto};
use crate::error::{ApiError, ApiResult};
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use domus_common::Error;
use domus_domain::services::auth::{LoginOutcome, OAuthProfile};
use reqwest::Url;
use serde::Deserialize;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth/authorize", post(authorize))
        .route("/oauth/backchannel-logout", post(backchannel_logout))
        .route("/oauth/callback", post(callback))
        .route("/oauth/link", post(link_account))
        .route("/oauth/mobile-redirect", get(mobile_redirect))
        .route("/oauth/unlink", post(unlink_account))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuthAuthorizeDto {
    redirect_uri: Option<String>,
    state: Option<String>,
}

async fn authorize(
    State(state): State<AppState>,
    Json(dto): Json<OAuthAuthorizeDto>,
) -> ApiResult<Json<serde_json::Value>> {
    let oauth = oauth_config(&state).await?;
    ensure_enabled(&oauth)?;
    let authorize_url = str_field(&oauth, "authorizeUrl")?;
    let client_id = str_field(&oauth, "clientId")?;
    let scope = oauth
        .get("scope")
        .and_then(|v| v.as_str())
        .unwrap_or("openid email profile");
    let redirect_uri = dto.redirect_uri.unwrap_or_default();
    let state_value = dto.state.unwrap_or_default();
    let url = Url::parse_with_params(
        authorize_url,
        [
            ("response_type", "code"),
            ("client_id", client_id),
            ("redirect_uri", redirect_uri.as_str()),
            ("scope", scope),
            ("state", state_value.as_str()),
        ],
    )
    .map_err(|e| {
        ApiError(Error::BadRequest(format!(
            "invalid OAuth authorizeUrl: {e}"
        )))
    })?;
    Ok(Json(serde_json::json!({ "url": url.as_str() })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuthCallbackDto {
    code: String,
    redirect_uri: Option<String>,
    #[serde(rename = "state")]
    _state: Option<String>,
}

async fn callback(
    State(state): State<AppState>,
    Json(dto): Json<OAuthCallbackDto>,
) -> ApiResult<(StatusCode, HeaderMap, Json<LoginResponseDto>)> {
    let oauth = oauth_config(&state).await?;
    ensure_enabled(&oauth)?;
    let profile = exchange_profile(&oauth, &dto).await?;
    let outcome = state.services.auth.oauth_login(profile).await?;
    Ok(login_response(outcome))
}

async fn link_account(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<OAuthCallbackDto>,
) -> ApiResult<Json<UserAdminResponseDto>> {
    let oauth = oauth_config(&state).await?;
    ensure_enabled(&oauth)?;
    let profile = exchange_profile(&oauth, &dto).await?;
    let user = state
        .services
        .auth
        .link_oauth_account(ctx.user_id, &profile.oauth_id)
        .await?;
    Ok(Json((&user).into()))
}

async fn unlink_account(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<UserAdminResponseDto>> {
    let user = state
        .services
        .auth
        .link_oauth_account(ctx.user_id, "")
        .await?;
    Ok(Json((&user).into()))
}

async fn backchannel_logout() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "successful": true }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MobileRedirectQuery {
    url: Option<String>,
}

async fn mobile_redirect(Query(query): Query<MobileRedirectQuery>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "url": query.url.unwrap_or_default() }))
}

async fn oauth_config(state: &AppState) -> ApiResult<serde_json::Value> {
    Ok(state
        .services
        .system_config
        .get()
        .await?
        .get("oauth")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({ "enabled": false })))
}

fn ensure_enabled(oauth: &serde_json::Value) -> ApiResult<()> {
    if oauth
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(ApiError(Error::BadRequest("OAuth is disabled".into())))
    }
}

async fn exchange_profile(
    oauth: &serde_json::Value,
    dto: &OAuthCallbackDto,
) -> ApiResult<OAuthProfile> {
    let token_endpoint = str_field(oauth, "tokenEndpoint")?;
    let client_id = str_field(oauth, "clientId")?;
    let client_secret = oauth
        .get("clientSecret")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut form = vec![
        ("grant_type", "authorization_code".to_owned()),
        ("code", dto.code.clone()),
        ("client_id", client_id.to_owned()),
    ];
    if !client_secret.is_empty() {
        form.push(("client_secret", client_secret.to_owned()));
    }
    if let Some(redirect_uri) = &dto.redirect_uri {
        form.push(("redirect_uri", redirect_uri.clone()));
    }

    let client = reqwest::Client::new();
    let token_body: serde_json::Value = client
        .post(token_endpoint)
        .form(&form)
        .send()
        .await
        .map_err(|e| ApiError(Error::Unauthorized(e.to_string())))?
        .error_for_status()
        .map_err(|e| ApiError(Error::Unauthorized(e.to_string())))?
        .json()
        .await
        .map_err(|e| ApiError(Error::Unauthorized(e.to_string())))?;

    if let Some(userinfo_endpoint) = oauth.get("userinfoEndpoint").and_then(|v| v.as_str()) {
        if let Some(access_token) = token_body.get("access_token").and_then(|v| v.as_str()) {
            let userinfo: serde_json::Value = client
                .get(userinfo_endpoint)
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| ApiError(Error::Unauthorized(e.to_string())))?
                .error_for_status()
                .map_err(|e| ApiError(Error::Unauthorized(e.to_string())))?
                .json()
                .await
                .map_err(|e| ApiError(Error::Unauthorized(e.to_string())))?;
            return profile_from_claims(&userinfo);
        }
    }

    if let Some(id_token) = token_body.get("id_token").and_then(|v| v.as_str()) {
        return profile_from_claims(&decode_jwt_payload(id_token)?);
    }

    Err(ApiError(Error::Unauthorized(
        "OAuth response did not include user profile claims".into(),
    )))
}

fn profile_from_claims(claims: &serde_json::Value) -> ApiResult<OAuthProfile> {
    let oauth_id = claims
        .get("sub")
        .and_then(|v| v.as_str())
        .or_else(|| claims.get("id").and_then(|v| v.as_str()))
        .ok_or_else(|| ApiError(Error::Unauthorized("OAuth profile missing subject".into())))?;
    let email = claims
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError(Error::Unauthorized("OAuth profile missing email".into())))?;
    let name = claims
        .get("name")
        .and_then(|v| v.as_str())
        .or_else(|| claims.get("preferred_username").and_then(|v| v.as_str()))
        .unwrap_or(email);
    Ok(OAuthProfile {
        oauth_id: oauth_id.to_owned(),
        email: email.to_owned(),
        name: name.to_owned(),
    })
}

fn decode_jwt_payload(id_token: &str) -> ApiResult<serde_json::Value> {
    let payload = id_token
        .split('.')
        .nth(1)
        .ok_or_else(|| ApiError(Error::Unauthorized("invalid id_token".into())))?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| ApiError(Error::Unauthorized("invalid id_token payload".into())))?;
    serde_json::from_slice(&bytes)
        .map_err(|_| ApiError(Error::Unauthorized("invalid id_token json".into())))
}

fn str_field<'a>(value: &'a serde_json::Value, field: &str) -> ApiResult<&'a str> {
    value
        .get(field)
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            ApiError(Error::BadRequest(format!(
                "OAuth {field} is not configured"
            )))
        })
}

fn login_response(outcome: LoginOutcome) -> (StatusCode, HeaderMap, Json<LoginResponseDto>) {
    let mut headers = HeaderMap::new();
    for cookie in [
        format!(
            "immich_access_token={}; Path=/; HttpOnly; SameSite=Lax",
            outcome.token
        ),
        "immich_auth_type=oauth; Path=/; HttpOnly; SameSite=Lax".to_string(),
        "immich_is_authenticated=true; Path=/; SameSite=Lax".to_string(),
    ] {
        headers.append(SET_COOKIE, cookie.parse().unwrap());
    }
    let user = outcome.user;
    (
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
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_from_claims_accepts_immich_oauth_userinfo_shape() {
        let profile = profile_from_claims(&serde_json::json!({
            "sub": "oauth-user-1",
            "email": "oauth@example.com",
            "name": "OAuth User"
        }))
        .unwrap();
        assert_eq!(profile.oauth_id, "oauth-user-1");
        assert_eq!(profile.email, "oauth@example.com");
        assert_eq!(profile.name, "OAuth User");
    }

    #[test]
    fn profile_from_claims_falls_back_to_id_and_preferred_username() {
        let profile = profile_from_claims(&serde_json::json!({
            "id": "provider-id",
            "email": "oauth@example.com",
            "preferred_username": "oauth-user"
        }))
        .unwrap();
        assert_eq!(profile.oauth_id, "provider-id");
        assert_eq!(profile.name, "oauth-user");
    }

    #[test]
    fn profile_from_claims_requires_subject_and_email() {
        assert!(profile_from_claims(&serde_json::json!({"email": "oauth@example.com"})).is_err());
        assert!(profile_from_claims(&serde_json::json!({"sub": "oauth-user-1"})).is_err());
    }

    #[test]
    fn decode_jwt_payload_reads_id_token_claims() {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"none"}"#);
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(r#"{"sub":"oauth-user-1","email":"oauth@example.com","name":"OAuth User"}"#);
        let claims = decode_jwt_payload(&format!("{header}.{payload}.")).unwrap();
        assert_eq!(claims["sub"], "oauth-user-1");
        assert_eq!(claims["email"], "oauth@example.com");
    }
}
