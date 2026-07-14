//! Immich-compatible routes: API key management.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api-keys", get(super::not_implemented).post(super::not_implemented))
        .route("/api-keys/me", get(super::not_implemented))
        .route("/api-keys/{id}", delete(super::not_implemented).get(super::not_implemented).put(super::not_implemented))
}
