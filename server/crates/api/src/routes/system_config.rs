//! Immich-compatible routes: admin system configuration.

use crate::error::ApiResult;
use crate::extractors::{AdminAuth, Auth};
use crate::state::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use domus_domain::services::system_config::default_config;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/system-config", get(get_config).put(update_config))
        .route("/system-config/defaults", get(get_defaults))
        .route("/system-config/storage-template-options", get(storage_template_options))
}

async fn get_config(
    State(state): State<AppState>,
    Auth(_): Auth,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.services.system_config.get().await?))
}

async fn update_config(
    State(state): State<AppState>,
    AdminAuth(_): AdminAuth,
    Json(config): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.services.system_config.set(config).await?))
}

async fn get_defaults(Auth(_): Auth) -> Json<serde_json::Value> {
    Json(default_config())
}

async fn storage_template_options(Auth(_): Auth) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "enabled": true,
        "hashVerificationEnabled": true,
        "template": "{{y}}/{{MM}}/{{filename}}",
        "yearOptions": ["{{y}}", "{{yyyy}}"],
        "monthOptions": ["{{MM}}"],
        "dayOptions": ["{{dd}}"],
        "hourOptions": [],
        "minuteOptions": [],
        "secondOptions": [],
        "presetOptions": [
            "{{y}}/{{MM}}/{{filename}}",
            "{{y}}/{{MM}}/{{dd}}/{{filename}}",
            "{{y}}/{{MM}}/{{name}}-{{assetId}}.{{ext}}"
        ],
    }))
}
