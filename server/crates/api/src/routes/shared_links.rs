//! Immich-compatible routes: public share links.

use crate::dto::{AssetResponseDto, SharedLinkResponseDto};
use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use base64::Engine;
use chrono::{DateTime, Utc};
use domus_common::types::SharedLinkType;
use serde::Deserialize;
use uuid::Uuid;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shared-links", get(list_shared_links).post(create_shared_link))
        .route("/shared-links/login", post(shared_link_login))
        .route("/shared-links/me", get(get_my_shared_link))
        .route("/shared-links/{id}", delete(remove_shared_link).get(get_shared_link).patch(update_shared_link))
        .route("/shared-links/{id}/assets", delete(remove_shared_link_assets).put(add_shared_link_assets))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SharedLinkQuery {
    key: Option<String>,
    slug: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSharedLinkDto {
    #[serde(default = "default_shared_link_type")]
    #[serde(rename = "type")]
    link_type: SharedLinkType,
    album_id: Option<Uuid>,
    #[serde(default)]
    asset_ids: Vec<Uuid>,
    description: Option<String>,
    password: Option<String>,
    slug: Option<String>,
    #[serde(default = "default_true")]
    allow_upload: bool,
    #[serde(default = "default_true")]
    allow_download: bool,
    #[serde(default = "default_true")]
    #[serde(rename = "showMetadata", alias = "showExif")]
    show_metadata: bool,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateSharedLinkDto {
    allow_upload: Option<bool>,
    allow_download: Option<bool>,
    #[serde(rename = "showMetadata", alias = "showExif")]
    show_metadata: Option<bool>,
    description: Option<String>,
    password: Option<String>,
    slug: Option<String>,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SharedLinkAssetsDto {
    asset_ids: Vec<Uuid>,
}

async fn list_shared_links(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<Vec<SharedLinkResponseDto>>> {
    let links = state.services.shared_link.list(ctx.user_id).await?;
    let mut response = Vec::with_capacity(links.len());
    for link in links {
        response.push(shared_link_response(&state, &link).await?);
    }
    Ok(Json(response))
}

async fn create_shared_link(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<CreateSharedLinkDto>,
) -> ApiResult<Json<SharedLinkResponseDto>> {
    let link = state
        .services
        .shared_link
        .create(
            ctx.user_id,
            dto.link_type,
            dto.album_id,
            &dto.asset_ids,
            dto.description,
            dto.password,
            dto.slug,
            dto.allow_upload,
            dto.allow_download,
            dto.show_metadata,
            dto.expires_at,
        )
        .await?;
    Ok(Json(shared_link_response(&state, &link).await?))
}

async fn shared_link_login(
    State(state): State<AppState>,
    Query(query): Query<SharedLinkQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let link = state
        .services
        .shared_link
        .resolve(query.key.as_deref(), query.slug.as_deref())
        .await?;
    let key = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&link.key);
    Ok(Json(serde_json::json!({
        "accessToken": key.clone(),
        "key": key,
        "id": link.id,
    })))
}

async fn get_my_shared_link(
    State(state): State<AppState>,
    Query(query): Query<SharedLinkQuery>,
) -> ApiResult<Json<SharedLinkResponseDto>> {
    let link = state
        .services
        .shared_link
        .resolve(query.key.as_deref(), query.slug.as_deref())
        .await?;
    Ok(Json(shared_link_response(&state, &link).await?))
}

async fn get_shared_link(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SharedLinkResponseDto>> {
    let link = state.services.shared_link.get(id).await?;
    Ok(Json(shared_link_response(&state, &link).await?))
}

async fn update_shared_link(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateSharedLinkDto>,
) -> ApiResult<Json<SharedLinkResponseDto>> {
    let link = state
        .services
        .shared_link
        .update_options(
            id,
            dto.allow_upload,
            dto.allow_download,
            dto.show_metadata,
            dto.description.as_deref(),
            dto.password.as_deref(),
            dto.slug.as_deref(),
            dto.expires_at,
        )
        .await?;
    Ok(Json(shared_link_response(&state, &link).await?))
}

async fn add_shared_link_assets(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<SharedLinkAssetsDto>,
) -> ApiResult<StatusCode> {
    state
        .services
        .shared_link
        .add_assets(id, &dto.asset_ids)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_shared_link_assets(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<SharedLinkAssetsDto>,
) -> ApiResult<StatusCode> {
    state
        .services
        .shared_link
        .remove_assets(id, &dto.asset_ids)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_shared_link(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.services.shared_link.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn shared_link_response(
    state: &AppState,
    link: &domus_db::entities::SharedLink,
) -> ApiResult<SharedLinkResponseDto> {
    let assets = state.services.shared_link.assets(link.id).await?;
    Ok(SharedLinkResponseDto::from_link(
        link,
        assets
            .iter()
            .map(|asset| AssetResponseDto::from_asset(asset, None))
            .collect(),
    ))
}

fn default_true() -> bool {
    true
}

fn default_shared_link_type() -> SharedLinkType {
    SharedLinkType::Individual
}
