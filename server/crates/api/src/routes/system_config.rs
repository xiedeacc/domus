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
    Json(storage_template_options_payload())
}

fn storage_template_options_payload() -> serde_json::Value {
    serde_json::json!({
        "yearOptions": ["y", "yy"],
        "monthOptions": ["M", "MM", "MMM", "MMMM"],
        "weekOptions": ["W", "WW"],
        "dayOptions": ["d", "dd"],
        "hourOptions": ["h", "hh", "H", "HH"],
        "minuteOptions": ["m", "mm"],
        "secondOptions": ["s", "ss", "SSS"],
        "presetOptions": [
            "{{y}}/{{y}}-{{MM}}-{{dd}}/{{filename}}",
            "{{y}}/{{MM}}-{{dd}}/{{filename}}",
            "{{y}}/{{MMMM}}-{{dd}}/{{filename}}",
            "{{y}}/{{MM}}/{{filename}}",
            "{{y}}/{{#if album}}{{album}}{{else}}Other/{{MM}}{{/if}}/{{filename}}",
            "{{#if album}}{{album-startDate-y}}/{{album}}{{else}}{{y}}/Other/{{MM}}{{/if}}/{{filename}}",
            "{{y}}/{{MMM}}/{{filename}}",
            "{{y}}/{{MMMM}}/{{filename}}",
            "{{y}}/{{MM}}/{{dd}}/{{filename}}",
            "{{y}}/{{MMMM}}/{{dd}}/{{filename}}",
            "{{y}}/{{y}}-{{MM}}/{{y}}-{{MM}}-{{dd}}/{{filename}}",
            "{{y}}-{{MM}}-{{dd}}/{{filename}}",
            "{{y}}-{{MMM}}-{{dd}}/{{filename}}",
            "{{y}}-{{MMMM}}-{{dd}}/{{filename}}",
            "{{y}}/{{y}}-{{MM}}/{{filename}}",
            "{{y}}/{{y}}-{{WW}}/{{filename}}",
            "{{y}}/{{y}}-{{MM}}-{{dd}}/{{assetId}}",
            "{{y}}/{{y}}-{{MM}}/{{assetId}}",
            "{{y}}/{{y}}-{{WW}}/{{assetId}}",
            "{{album}}/{{filename}}",
            "{{make}}/{{model}}/{{lensModel}}/{{filename}}"
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_template_options_match_immich_tokens_and_presets() {
        let options = storage_template_options_payload();
        assert_eq!(options["yearOptions"], serde_json::json!(["y", "yy"]));
        assert_eq!(
            options["monthOptions"],
            serde_json::json!(["M", "MM", "MMM", "MMMM"])
        );
        assert_eq!(options["weekOptions"], serde_json::json!(["W", "WW"]));
        assert_eq!(options["dayOptions"], serde_json::json!(["d", "dd"]));
        assert_eq!(
            options["hourOptions"],
            serde_json::json!(["h", "hh", "H", "HH"])
        );
        assert_eq!(options["minuteOptions"], serde_json::json!(["m", "mm"]));
        assert_eq!(
            options["secondOptions"],
            serde_json::json!(["s", "ss", "SSS"])
        );
        assert_eq!(options["presetOptions"].as_array().map(Vec::len), Some(21));
        assert!(options["presetOptions"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("{{y}}/{{y}}-{{WW}}/{{filename}}")));
    }
}
