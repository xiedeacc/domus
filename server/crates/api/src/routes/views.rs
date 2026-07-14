//! Immich-compatible routes: folder view of original paths.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/view/folder", get(super::not_implemented))
        .route("/view/folder/unique-paths", get(super::not_implemented))
}
