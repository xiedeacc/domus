//! Immich-compatible routes: admin job control.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs", get(super::not_implemented).post(super::not_implemented))
        .route("/jobs/{name}", put(super::not_implemented))
}
