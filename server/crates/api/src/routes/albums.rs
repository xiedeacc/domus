//! /albums — CRUD, album assets, album users (sharing), statistics.

use crate::dto::{AlbumResponseDto, AssetResponseDto};
use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/albums", get(list_albums).post(create_album))
        .route("/albums/statistics", get(super::not_implemented))
        .route(
            "/albums/{id}",
            get(get_album)
                .patch(super::not_implemented)
                .delete(delete_album),
        )
        .route("/albums/{id}/assets", put(add_assets).delete(remove_assets))
        .route("/albums/{id}/users", put(add_users))
        .route(
            "/albums/{id}/user/{userId}",
            put(update_user).delete(remove_user),
        )
        .route("/albums/{id}/slideshow", get(super::not_implemented))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListAlbumsQuery {
    shared: Option<bool>,
    #[allow(dead_code)]
    asset_id: Option<Uuid>,
}

async fn list_albums(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Query(q): Query<ListAlbumsQuery>,
) -> ApiResult<Json<Vec<AlbumResponseDto>>> {
    let albums = state.services.album.list(ctx.user_id, q.shared).await?;
    let mut response = Vec::with_capacity(albums.len());
    for album in albums {
        let count = state.services.album.asset_count(album.id).await?;
        response.push(AlbumResponseDto::from_album(&album, count, vec![]));
    }
    Ok(Json(response))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateAlbumDto {
    album_name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    asset_ids: Vec<Uuid>,
    #[serde(default)]
    #[allow(dead_code)]
    album_users: Vec<serde_json::Value>,
}

async fn create_album(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<CreateAlbumDto>,
) -> ApiResult<(StatusCode, Json<AlbumResponseDto>)> {
    let album = state
        .services
        .album
        .create(ctx.user_id, &dto.album_name, &dto.description)
        .await?;
    if !dto.asset_ids.is_empty() {
        state
            .services
            .album
            .add_assets(album.id, &dto.asset_ids)
            .await?;
    }
    let count = state.services.album.asset_count(album.id).await?;
    Ok((
        StatusCode::CREATED,
        Json(AlbumResponseDto::from_album(&album, count, vec![])),
    ))
}

async fn get_album(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AlbumResponseDto>> {
    let album = state.services.album.get(id).await?;
    let assets = state.services.album.assets(id).await?;
    let mut asset_values = Vec::with_capacity(assets.len());
    for asset in assets {
        let exif = state.services.asset.get(asset.id).await?.1;
        asset_values.push(
            serde_json::to_value(AssetResponseDto::from_asset(&asset, exif.as_ref())).unwrap(),
        );
    }
    let count = asset_values.len() as i64;
    Ok(Json(AlbumResponseDto::from_album(
        &album,
        count,
        asset_values,
    )))
}

async fn delete_album(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.services.album.delete(id).await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct BulkIdsDto {
    ids: Vec<Uuid>,
}

async fn add_assets(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<BulkIdsDto>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.album.add_assets(id, &dto.ids).await?;
    Ok(Json(serde_json::json!([])))
}

async fn remove_assets(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<BulkIdsDto>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.album.remove_assets(id, &dto.ids).await?;
    Ok(Json(serde_json::json!([])))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlbumUserDto {
    user_id: Uuid,
    #[serde(default = "default_album_role")]
    role: String,
}

async fn add_users(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<Vec<AlbumUserDto>>,
) -> ApiResult<Json<serde_json::Value>> {
    let users: Vec<_> = dto.into_iter().map(|u| (u.user_id, u.role)).collect();
    state.services.album.add_users(id, &users).await?;
    Ok(Json(serde_json::json!([])))
}

async fn update_user(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
    Json(dto): Json<serde_json::Value>,
) -> ApiResult<StatusCode> {
    let role = dto.get("role").and_then(|v| v.as_str()).unwrap_or("editor");
    state.services.album.update_user(id, user_id, role).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_user(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    state.services.album.remove_user(id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn default_album_role() -> String {
    "editor".to_owned()
}
