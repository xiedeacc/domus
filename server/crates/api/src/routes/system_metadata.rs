//! Immich-compatible routes: admin system metadata flags.

use crate::error::ApiResult;
use crate::state::AppState;
use axum::extract::State;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::{Json, Router};

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/system-metadata/admin-onboarding", get(super::not_implemented).post(super::not_implemented))
        .route("/system-metadata/reverse-geocoding-state", get(super::not_implemented))
        .route("/system-metadata/version-check-state", get(version_check_state))
}

async fn version_check_state(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(super::server::version_check_value(&state)))
}
