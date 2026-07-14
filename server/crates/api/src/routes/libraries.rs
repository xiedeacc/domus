//! Immich-compatible routes: external library management.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/libraries", get(super::not_implemented).post(super::not_implemented))
        .route("/libraries/{id}", delete(super::not_implemented).get(super::not_implemented).put(super::not_implemented))
        .route("/libraries/{id}/scan", post(super::not_implemented))
        .route("/libraries/{id}/statistics", get(super::not_implemented))
        .route("/libraries/{id}/validate", post(super::not_implemented))
}
