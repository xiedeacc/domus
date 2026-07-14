//! Immich-compatible routes: API key management.

use crate::error::{ApiError, ApiResult};
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};
use chrono::{DateTime, SecondsFormat, Utc};
use domus_common::Error;
use domus_db::entities::ApiKey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api-keys", get(list_api_keys).post(create_api_key))
        .route("/api-keys/me", get(get_my_api_key))
        .route("/api-keys/{id}", delete(delete_api_key).get(get_api_key).put(super::not_implemented))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiKeyResponseDto {
    id: Uuid,
    name: String,
    created_at: String,
    updated_at: String,
    permissions: Vec<String>,
}

impl From<&ApiKey> for ApiKeyResponseDto {
    fn from(key: &ApiKey) -> Self {
        Self {
            id: key.id,
            name: key.name.clone(),
            created_at: iso(&key.created_at),
            updated_at: iso(&key.updated_at),
            permissions: key.permissions.clone(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreatedApiKeyResponseDto {
    #[serde(flatten)]
    api_key: ApiKeyResponseDto,
    secret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateApiKeyDto {
    #[serde(default = "default_key_name")]
    name: String,
    #[serde(default)]
    permissions: Vec<String>,
}

async fn list_api_keys(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<Vec<ApiKeyResponseDto>>> {
    let keys = state.services.auth.list_api_keys(ctx.user_id).await?;
    Ok(Json(keys.iter().map(Into::into).collect()))
}

async fn create_api_key(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<CreateApiKeyDto>,
) -> ApiResult<(StatusCode, Json<CreatedApiKeyResponseDto>)> {
    let outcome = state
        .services
        .auth
        .create_api_key(ctx.user_id, &dto.name, &dto.permissions)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedApiKeyResponseDto {
            api_key: (&outcome.api_key).into(),
            secret: outcome.secret,
        }),
    ))
}

async fn get_my_api_key(Auth(ctx): Auth) -> ApiResult<Json<serde_json::Value>> {
    let id = ctx
        .api_key_id
        .ok_or_else(|| ApiError(Error::NotFound("API key not found".into())))?;
    Ok(Json(serde_json::json!({ "id": id })))
}

async fn get_api_key(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ApiKeyResponseDto>> {
    let keys = state.services.auth.list_api_keys(ctx.user_id).await?;
    let key = keys
        .iter()
        .find(|key| key.id == id)
        .ok_or_else(|| ApiError(Error::NotFound("API key not found".into())))?;
    Ok(Json(key.into()))
}

async fn delete_api_key(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.services.auth.delete_api_key(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn default_key_name() -> String {
    "API Key".to_owned()
}

fn iso(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}
