//! /albums — CRUD, album assets, album users (sharing), statistics.

use crate::dto::AlbumResponseDto;
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
            get(get_album).patch(super::not_implemented).delete(delete_album),
        )
        .route("/albums/{id}/assets", put(add_assets).delete(remove_assets))
        .route("/albums/{id}/users", put(super::not_implemented))
        .route("/albums/{id}/user/{userId}", put(super::not_implemented).delete(super::not_implemented))
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
    Ok(Json(albums.iter().map(Into::into).collect()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateAlbumDto {
    album_name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    #[allow(dead_code)]
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
    Ok((StatusCode::CREATED, Json((&album).into())))
}

async fn get_album(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AlbumResponseDto>> {
    let album = state.services.album.get(id).await?;
    Ok(Json((&album).into()))
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
