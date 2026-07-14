//! Immich-compatible routes: admin system configuration.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/system-config", get(super::not_implemented).put(super::not_implemented))
        .route("/system-config/defaults", get(super::not_implemented))
        .route("/system-config/storage-template-options", get(super::not_implemented))
}
