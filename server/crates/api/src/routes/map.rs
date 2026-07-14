//! Immich-compatible routes: map markers and reverse geocoding.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/map/markers", get(super::not_implemented))
        .route("/map/reverse-geocode", get(super::not_implemented))
}
