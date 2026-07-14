//! Immich-compatible routes: metadata/people/places search; smart search 501s without ML.
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search/cities", get(super::not_implemented))
        .route("/search/explore", get(super::not_implemented))
        .route("/search/large-assets", post(super::not_implemented))
        .route("/search/metadata", post(super::not_implemented))
        .route("/search/person", get(super::not_implemented))
        .route("/search/places", get(super::not_implemented))
        .route("/search/random", post(super::not_implemented))
        .route("/search/smart", post(super::not_implemented))
        .route("/search/statistics", post(super::not_implemented))
        .route("/search/suggestions", get(super::not_implemented))
}
