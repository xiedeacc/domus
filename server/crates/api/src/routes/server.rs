//! /server — version, features, config, about, ping, storage, statistics.
//! These are the first endpoints every client probes, so they are fully
//! implemented in the skeleton.

use crate::error::ApiResult;
use crate::extractors::{AdminAuth, Auth};
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/server/about", get(about))
        .route("/server/apk-links", get(super::not_implemented))
        .route("/server/config", get(config))
        .route("/server/features", get(features))
        .route(
            "/server/license",
            get(authenticated_not_implemented)
                .put(authenticated_not_implemented)
                .delete(authenticated_not_implemented),
        )
        .route("/server/media-types", get(media_types))
        .route("/server/ping", get(ping))
        .route("/server/statistics", get(statistics))
        .route("/server/storage", get(storage))
        .route("/server/theme", get(theme))
        .route("/server/version", get(version))
        .route("/server/version-check", get(version_check))
        .route("/server/version-history", get(version_history))
}

async fn ping() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "res": "pong" }))
}

async fn version(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(state.services.server.version())
}

async fn version_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(version_check_value(&state))
}

async fn version_history() -> Json<serde_json::Value> {
    Json(serde_json::json!([]))
}

async fn authenticated_not_implemented(Auth(_): Auth) -> crate::error::ApiError {
    super::not_implemented().await
}

async fn features(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(state.services.server.features())
}

async fn config(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.services.server.config().await?))
}

async fn about(State(state): State<AppState>, Auth(_): Auth) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.services.server.about().await?))
}

async fn storage(
    State(state): State<AppState>,
    Auth(_): Auth,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.services.server.storage().await?))
}

async fn statistics(
    State(_): State<AppState>,
    AdminAuth(_): AdminAuth,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "photos": 0, "videos": 0, "usage": 0,
        "usagePhotos": 0, "usageVideos": 0, "usageByUser": []
    })))
}

async fn theme() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "customCss": "" }))
}

async fn media_types() -> Json<serde_json::Value> {
    Json(media_types_value())
}

fn media_types_value() -> serde_json::Value {
    serde_json::json!({
        "image": [".arw", ".avif", ".bmp", ".cr2", ".cr3", ".dng", ".gif", ".heic", ".heif", ".jpeg", ".jpg", ".nef", ".orf", ".pef", ".png", ".raf", ".raw", ".rw2", ".srw", ".tiff", ".webp"],
        "video": [".3gp", ".avi", ".flv", ".m2ts", ".mkv", ".mov", ".mp4", ".mpg", ".mts", ".webm", ".wmv"],
        "sidecar": [".xmp"],
    })
}

pub(crate) fn version_check_value(state: &AppState) -> serde_json::Value {
    let about = state.services.server.version();
    let major = about["major"].as_u64().unwrap_or(3);
    let minor = about["minor"].as_u64().unwrap_or(0);
    let patch = about["patch"].as_u64().unwrap_or(3);
    serde_json::json!({
        "checkedAt": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        "releaseVersion": format!("v{major}.{minor}.{patch}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_types_include_immich_image_video_and_sidecar_groups() {
        let value = media_types_value();
        assert!(value["image"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(".heic")));
        assert!(value["image"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(".dng")));
        assert!(value["video"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(".mp4")));
        assert!(value["video"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(".mov")));
        assert_eq!(value["sidecar"], serde_json::json!([".xmp"]));
    }

    #[tokio::test]
    async fn ping_matches_immich_pong_shape() {
        assert_eq!(ping().await.0, serde_json::json!({ "res": "pong" }));
    }
}
