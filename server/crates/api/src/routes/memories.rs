//! Immich-compatible routes: "on this day" memories.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/memories", get(super::not_implemented).post(super::not_implemented))
        .route("/memories/statistics", get(super::not_implemented))
        .route("/memories/{id}", delete(super::not_implemented).get(super::not_implemented).put(super::not_implemented))
        .route("/memories/{id}/assets", delete(super::not_implemented).put(super::not_implemented))
}
