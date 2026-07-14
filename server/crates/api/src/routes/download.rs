//! Immich-compatible routes: bulk download (zip archives).
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/download/archive", post(super::not_implemented))
        .route("/download/info", post(super::not_implemented))
}
