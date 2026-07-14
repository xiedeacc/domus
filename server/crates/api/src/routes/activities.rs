//! Immich-compatible routes: comments & likes on shared album assets.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/activities", get(super::not_implemented).post(super::not_implemented))
        .route("/activities/statistics", get(super::not_implemented))
        .route("/activities/{id}", delete(super::not_implemented))
}
