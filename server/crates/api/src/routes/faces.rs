//! Immich-compatible routes: face-asset assignment (data model served; detection needs ML and is disabled).
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/faces", get(super::not_implemented).post(super::not_implemented))
        .route("/faces/{id}", delete(super::not_implemented).put(super::not_implemented))
}
