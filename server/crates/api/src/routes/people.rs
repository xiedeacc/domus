//! Immich-compatible routes: people albums (recognition itself needs ML and is disabled).
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/people", delete(super::not_implemented).get(super::not_implemented).post(super::not_implemented).put(super::not_implemented))
        .route("/people/{id}", delete(super::not_implemented).get(super::not_implemented).put(super::not_implemented))
        .route("/people/{id}/merge", post(super::not_implemented))
        .route("/people/{id}/reassign", put(super::not_implemented))
        .route("/people/{id}/statistics", get(super::not_implemented))
        .route("/people/{id}/thumbnail", get(super::not_implemented))
}
