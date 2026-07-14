//! Immich-compatible routes: admin system metadata flags.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/system-metadata/admin-onboarding", get(super::not_implemented).post(super::not_implemented))
        .route("/system-metadata/reverse-geocoding-state", get(super::not_implemented))
        .route("/system-metadata/version-check-state", get(super::not_implemented))
}
