//! Immich-compatible routes: trash restore/empty.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trash/empty", post(super::not_implemented))
        .route("/trash/restore", post(super::not_implemented))
        .route("/trash/restore/assets", post(super::not_implemented))
}
