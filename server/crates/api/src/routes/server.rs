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
        .route("/server/license", get(super::not_implemented).put(super::not_implemented).delete(super::not_implemented))
        .route("/server/media-types", get(media_types))
        .route("/server/ping", get(ping))
        .route("/server/statistics", get(statistics))
        .route("/server/storage", get(storage))
        .route("/server/theme", get(theme))
        .route("/server/version", get(version))
        .route("/server/version-check", get(super::not_implemented))
        .route("/server/version-history", get(version_history))
}

async fn ping() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "res": "pong" }))
}

async fn version(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(state.services.server.version())
}

async fn version_history() -> Json<serde_json::Value> {
    Json(serde_json::json!([]))
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

async fn storage(State(state): State<AppState>, Auth(_): Auth) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.services.server.storage().await?))
}

async fn statistics(State(_): State<AppState>, AdminAuth(_): AdminAuth) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "photos": 0, "videos": 0, "usage": 0,
        "usagePhotos": 0, "usageVideos": 0, "usageByUser": []
    })))
}

async fn theme() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "customCss": "" }))
}

async fn media_types() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "image": [".avif", ".bmp", ".gif", ".heic", ".heif", ".jpeg", ".jpg", ".png", ".raw", ".tiff", ".webp"],
        "video": [".3gp", ".avi", ".flv", ".m2ts", ".mkv", ".mov", ".mp4", ".mpg", ".mts", ".webm", ".wmv"],
        "sidecar": [".xmp"],
    }))
}
