//! /assets — upload (multipart), metadata CRUD, original/thumbnail/video
//! serving, bulk operations, upload checks.

use crate::error::{ApiError, ApiResult};
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use domus_common::Error;
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/assets",
            post(upload_asset).put(super::not_implemented).delete(delete_assets),
        )
        .route("/assets/{id}", get(get_asset).put(super::not_implemented))
        .route("/assets/{id}/original", get(download_original))
        .route("/assets/{id}/thumbnail", get(view_thumbnail))
        .route("/assets/{id}/video/playback", get(play_video))
        .route("/assets/{id}/metadata", get(super::not_implemented).put(super::not_implemented))
        .route("/assets/{id}/ocr", get(super::not_implemented))
        .route("/assets/device/{deviceId}", get(get_all_user_assets_by_device_id))
        .route("/assets/exist", post(check_existing_assets))
        .route("/assets/bulk-upload-check", post(check_bulk_upload))
        .route("/assets/jobs", post(run_asset_jobs))
        .route("/assets/statistics", get(get_statistics))
        .route("/assets/random", get(super::not_implemented))
        .route("/assets/copy", post(super::not_implemented))
        .route("/assets/{id}/edits", put(super::not_implemented))
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SharedLinkQuery {
    key: Option<String>,
    slug: Option<String>,
}

/// POST /assets — multipart upload.
/// Fields: assetData (file), deviceAssetId, deviceId, fileCreatedAt,
/// fileModifiedAt, isFavorite, duration, sidecarData?; header
/// x-immich-checksum lets the server short-circuit duplicates.
/// Replies 201 {id, status:"created"} or 200 {id, status:"duplicate"}.
async fn upload_asset(
    State(_state): State<AppState>,
    Auth(_ctx): Auth,
    _multipart: Multipart,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    // TODO: stream parts to a staging file, compute SHA-1, then call
    // services.asset_media.upload(). See AssetMediaService::upload for flow.
    Err(ApiError(Error::NotImplemented("uploadAsset")))
}

async fn get_asset(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let (_asset, _exif) = state.services.asset.get(id).await?;
    // TODO: map to AssetResponseDto.
    Err(ApiError(Error::NotImplemented("getAssetInfo mapping")))
}

#[derive(Deserialize)]
struct AssetBulkDeleteDto {
    ids: Vec<Uuid>,
    #[serde(default)]
    force: bool,
}

async fn delete_assets(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Json(dto): Json<AssetBulkDeleteDto>,
) -> ApiResult<StatusCode> {
    state.services.asset.delete(&dto.ids, dto.force).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /assets/:id/original — stream the original file with range support.
async fn download_original(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<axum::response::Response> {
    let path = state.services.asset_media.original_path(id).await?;
    serve_file(path).await
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ThumbnailQuery {
    size: Option<String>, // "preview" | "thumbnail" | "fullsize"
}

async fn view_thumbnail(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
    Query(q): Query<ThumbnailQuery>,
) -> ApiResult<axum::response::Response> {
    let size = q.size.as_deref().unwrap_or("thumbnail");
    let path = state.services.asset_media.thumbnail_path(id, size).await?;
    serve_file(path).await
}

async fn play_video(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<axum::response::Response> {
    let path = state.services.asset_media.playback_path(id).await?;
    serve_file(path).await
}

async fn get_all_user_assets_by_device_id(
    Auth(_ctx): Auth,
    Path(_device_id): Path<String>,
) -> ApiResult<Json<Vec<String>>> {
    Err(ApiError(Error::NotImplemented("getAllUserAssetsByDeviceId")))
}

async fn check_existing_assets(Auth(_ctx): Auth) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError(Error::NotImplemented("checkExistingAssets")))
}

async fn check_bulk_upload(Auth(_ctx): Auth) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError(Error::NotImplemented("checkBulkUpload")))
}

async fn run_asset_jobs(Auth(_ctx): Auth) -> ApiResult<StatusCode> {
    Err(ApiError(Error::NotImplemented("runAssetJobs")))
}

async fn get_statistics(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<serde_json::Value>> {
    let (images, videos) = state.services.asset.statistics(ctx.user_id).await?;
    Ok(Json(serde_json::json!({
        "images": images, "videos": videos, "total": images + videos
    })))
}

/// Stream a file from disk honouring Range headers (tower-http ServeFile).
async fn serve_file(path: std::path::PathBuf) -> ApiResult<axum::response::Response> {
    use tower::ServiceExt;
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let svc = tower_http::services::ServeFile::new_with_mime(&path, &mime);
    let response = svc
        .oneshot(axum::http::Request::new(axum::body::Body::empty()))
        .await
        .map_err(|e| ApiError(Error::Internal(e.into())))?;
    Ok(response.map(axum::body::Body::new))
}
