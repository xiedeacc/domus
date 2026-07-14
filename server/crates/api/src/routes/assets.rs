//! /assets — upload (multipart), metadata CRUD, original/thumbnail/video
//! serving, bulk operations, upload checks.

use crate::dto::AssetResponseDto;
use crate::error::{ApiError, ApiResult};
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use base64::Engine;
use domus_common::Error;
use domus_domain::services::asset_media::{UploadOutcome, UploadRequest};
use domus_domain::services::auth::AuthContext;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/assets",
            post(upload_asset).put(update_assets).delete(delete_assets),
        )
        .route("/assets/{id}", get(get_asset).put(update_asset))
        .route("/assets/{id}/original", get(download_original))
        .route("/assets/{id}/thumbnail", get(view_thumbnail))
        .route("/assets/{id}/video/playback", get(play_video))
        .route(
            "/assets/{id}/metadata",
            get(super::not_implemented).put(super::not_implemented),
        )
        .route("/assets/{id}/ocr", get(super::not_implemented))
        .route(
            "/assets/device/{deviceId}",
            get(get_all_user_assets_by_device_id),
        )
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
    State(state): State<AppState>,
    Auth(ctx): Auth,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let mut parsed = ParsedUpload::default();
    let mut staged_file = None;
    let mut checksum = Sha1::new();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError(Error::BadRequest(e.to_string())))?
    {
        let name = field.name().unwrap_or_default().to_owned();
        match name.as_str() {
            "assetData" => {
                let filename = field
                    .file_name()
                    .map(str::to_owned)
                    .unwrap_or_else(|| "upload.bin".to_owned());
                let path = std::env::temp_dir().join(format!("domus-upload-{}", Uuid::new_v4()));
                let mut file = tokio::fs::File::create(&path)
                    .await
                    .map_err(|e| ApiError(Error::Io(e)))?;
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|e| ApiError(Error::BadRequest(e.to_string())))?
                {
                    checksum.update(&chunk);
                    file.write_all(&chunk)
                        .await
                        .map_err(|e| ApiError(Error::Io(e)))?;
                }
                parsed.filename = Some(filename);
                staged_file = Some(path);
            }
            "deviceAssetId" => parsed.device_asset_id = Some(field_text(field).await?),
            "deviceId" => parsed.device_id = Some(field_text(field).await?),
            "fileCreatedAt" => {
                parsed.file_created_at = Some(parse_datetime(&field_text(field).await?)?)
            }
            "fileModifiedAt" => {
                parsed.file_modified_at = Some(parse_datetime(&field_text(field).await?)?)
            }
            "isFavorite" => parsed.is_favorite = parse_bool(&field_text(field).await?),
            "livePhotoVideoId" => {
                parsed.live_photo_video_id = Some(parse_uuid(&field_text(field).await?)?)
            }
            _ => {}
        }
    }

    let computed_checksum = checksum.finalize().to_vec();
    let header_checksum = headers
        .get("x-immich-checksum")
        .and_then(|h| h.to_str().ok())
        .and_then(decode_checksum);
    let checksum = header_checksum.unwrap_or(computed_checksum);
    let now = chrono::Utc::now();
    let outcome = state
        .services
        .asset_media
        .upload(UploadRequest {
            owner_id: ctx.user_id,
            device_asset_id: parsed
                .device_asset_id
                .ok_or_else(|| ApiError(Error::BadRequest("deviceAssetId is required".into())))?,
            device_id: parsed.device_id.unwrap_or_default(),
            file_created_at: parsed.file_created_at.unwrap_or(now),
            file_modified_at: parsed.file_modified_at.unwrap_or(now),
            filename: parsed.filename.unwrap_or_else(|| "upload.bin".to_owned()),
            is_favorite: parsed.is_favorite.unwrap_or(false),
            live_photo_video_id: parsed.live_photo_video_id,
            staged_file: staged_file
                .ok_or_else(|| ApiError(Error::BadRequest("assetData is required".into())))?,
            checksum,
        })
        .await?;

    match outcome {
        UploadOutcome::Created(id) => Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": id, "status": "created" })),
        )),
        UploadOutcome::Duplicate(id) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "id": id, "status": "duplicate" })),
        )),
    }
}

async fn get_asset(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AssetResponseDto>> {
    ensure_asset_access(&state, &ctx, id, false).await?;
    let (asset, exif) = state.services.asset.get(id).await?;
    Ok(Json(AssetResponseDto::from_asset(&asset, exif.as_ref())))
}

async fn update_asset(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
    Json(update): Json<serde_json::Value>,
) -> ApiResult<Json<AssetResponseDto>> {
    let asset = state.services.asset.update(id, update).await?;
    let exif = state.services.asset.get(id).await?.1;
    Ok(Json(AssetResponseDto::from_asset(&asset, exif.as_ref())))
}

#[derive(Deserialize)]
struct AssetBulkUpdateDto {
    ids: Vec<Uuid>,
    #[serde(flatten)]
    update: serde_json::Value,
}

async fn update_assets(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Json(dto): Json<AssetBulkUpdateDto>,
) -> ApiResult<StatusCode> {
    state
        .services
        .asset
        .bulk_update(&dto.ids, dto.update)
        .await?;
    Ok(StatusCode::NO_CONTENT)
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
    Auth(ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<axum::response::Response> {
    ensure_asset_access(&state, &ctx, id, true).await?;
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
    Auth(ctx): Auth,
    Path(id): Path<Uuid>,
    Query(q): Query<ThumbnailQuery>,
) -> ApiResult<axum::response::Response> {
    ensure_asset_access(&state, &ctx, id, false).await?;
    let size = q.size.as_deref().unwrap_or("thumbnail");
    let path = state.services.asset_media.thumbnail_path(id, size).await?;
    serve_file(path).await
}

async fn play_video(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<axum::response::Response> {
    ensure_asset_access(&state, &ctx, id, true).await?;
    let path = state.services.asset_media.playback_path(id).await?;
    serve_file(path).await
}

async fn get_all_user_assets_by_device_id(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Path(device_id): Path<String>,
) -> ApiResult<Json<Vec<String>>> {
    let ids = state
        .services
        .asset
        .device_asset_ids(ctx.user_id, &device_id)
        .await?;
    Ok(Json(ids))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckExistingAssetsDto {
    device_asset_ids: Vec<String>,
}

async fn check_existing_assets(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<CheckExistingAssetsDto>,
) -> ApiResult<Json<serde_json::Value>> {
    let existing = state
        .services
        .asset_media
        .check_existing(ctx.user_id, &dto.device_asset_ids)
        .await?;
    Ok(Json(serde_json::json!({ "existingIds": existing })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BulkUploadCheckDto {
    assets: Vec<BulkUploadCheckAsset>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BulkUploadCheckAsset {
    id: Option<String>,
    device_asset_id: Option<String>,
}

async fn check_bulk_upload(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<BulkUploadCheckDto>,
) -> ApiResult<Json<serde_json::Value>> {
    let ids: Vec<String> = dto
        .assets
        .iter()
        .filter_map(|a| a.device_asset_id.clone().or_else(|| a.id.clone()))
        .collect();
    let existing = state
        .services
        .asset_media
        .check_existing(ctx.user_id, &ids)
        .await?;
    let results: Vec<_> = dto
        .assets
        .into_iter()
        .map(|asset| {
            let id = asset.device_asset_id.or(asset.id).unwrap_or_default();
            let exists = existing.contains(&id);
            serde_json::json!({
                "id": id,
                "action": if exists { "reject" } else { "accept" },
                "reason": if exists { serde_json::json!("duplicate") } else { serde_json::Value::Null },
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "results": results })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunAssetJobsDto {
    ids: Vec<Uuid>,
    name: String,
}

async fn run_asset_jobs(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Json(dto): Json<RunAssetJobsDto>,
) -> ApiResult<StatusCode> {
    state.services.asset.run_job(&dto.ids, &dto.name).await?;
    Ok(StatusCode::NO_CONTENT)
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

#[derive(Default)]
struct ParsedUpload {
    device_asset_id: Option<String>,
    device_id: Option<String>,
    file_created_at: Option<chrono::DateTime<chrono::Utc>>,
    file_modified_at: Option<chrono::DateTime<chrono::Utc>>,
    filename: Option<String>,
    is_favorite: Option<bool>,
    live_photo_video_id: Option<Uuid>,
}

async fn field_text(field: axum::extract::multipart::Field<'_>) -> ApiResult<String> {
    field
        .text()
        .await
        .map_err(|e| ApiError(Error::BadRequest(e.to_string())))
}

fn parse_datetime(value: &str) -> ApiResult<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| ApiError(Error::BadRequest(format!("invalid timestamp: {value}"))))
}

fn parse_uuid(value: &str) -> ApiResult<Uuid> {
    Uuid::parse_str(value)
        .map_err(|_| ApiError(Error::BadRequest(format!("invalid uuid: {value}"))))
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn decode_checksum(value: &str) -> Option<Vec<u8>> {
    hex::decode(value)
        .ok()
        .or_else(|| base64::engine::general_purpose::STANDARD.decode(value).ok())
        .filter(|bytes| bytes.len() == 20)
}

async fn ensure_asset_access(
    state: &AppState,
    ctx: &AuthContext,
    asset_id: Uuid,
    requires_download: bool,
) -> ApiResult<()> {
    if let Some(shared_link_id) = ctx.shared_link_id {
        let link = state.services.shared_link.get(shared_link_id).await?;
        if requires_download && !link.allow_download {
            return Err(ApiError(Error::Forbidden(
                "download disabled for shared link".into(),
            )));
        }
        let asset_ids = state.services.shared_link.asset_ids(shared_link_id).await?;
        if !asset_ids.contains(&asset_id) {
            return Err(ApiError(Error::Forbidden(
                "asset is not part of shared link".into(),
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bool_accepts_immich_multipart_values() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("yes"), None);
    }

    #[test]
    fn decode_checksum_accepts_hex_and_base64_sha1() {
        let bytes = [7u8; 20];
        assert_eq!(decode_checksum(&hex::encode(bytes)), Some(bytes.to_vec()));
        assert_eq!(
            decode_checksum(&base64::engine::general_purpose::STANDARD.encode(bytes)),
            Some(bytes.to_vec())
        );
        assert_eq!(decode_checksum("not-a-checksum"), None);
    }

    #[test]
    fn parse_datetime_requires_rfc3339() {
        let parsed = parse_datetime("2026-07-14T12:13:14.123Z").unwrap();
        assert_eq!(
            parsed.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "2026-07-14T12:13:14.123Z"
        );
        assert!(parse_datetime("2026-07-14").is_err());
    }
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
