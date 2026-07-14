//! Immich-compatible routes: public share links.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/shared-links", get(super::not_implemented).post(super::not_implemented))
        .route("/shared-links/login", post(super::not_implemented))
        .route("/shared-links/me", get(super::not_implemented))
        .route("/shared-links/{id}", delete(super::not_implemented).get(super::not_implemented).patch(super::not_implemented))
        .route("/shared-links/{id}/assets", delete(super::not_implemented).put(super::not_implemented))
}
