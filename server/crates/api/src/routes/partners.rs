//! Immich-compatible routes: partner sharing.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/partners", get(super::not_implemented).post(super::not_implemented))
        .route("/partners/{id}", delete(super::not_implemented).post(super::not_implemented).put(super::not_implemented))
}
