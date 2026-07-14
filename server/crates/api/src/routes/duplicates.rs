//! Immich-compatible routes: duplicate asset review (requires ML embeddings upstream; listing endpoints still served).
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/duplicates", delete(super::not_implemented).get(super::not_implemented))
        .route("/duplicates/resolve", post(super::not_implemented))
        .route("/duplicates/{id}", delete(super::not_implemented))
}
