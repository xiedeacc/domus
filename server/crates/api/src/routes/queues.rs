//! Immich-compatible routes: admin queue control.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/queues", get(super::not_implemented))
        .route("/queues/{name}", get(super::not_implemented).put(super::not_implemented))
        .route("/queues/{name}/jobs", delete(super::not_implemented).get(super::not_implemented))
}
