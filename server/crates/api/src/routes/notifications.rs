//! Immich-compatible routes: in-app notifications (user + admin templates).
//! Skeleton: every route is mounted; handlers answer 501 until implemented.

use crate::state::AppState;
#[allow(unused_imports)]
use axum::routing::{delete, get, patch, post, put};
use axum::Router;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/notifications", post(super::not_implemented))
        .route("/admin/notifications/templates/{name}", post(super::not_implemented))
        .route("/admin/notifications/test-email", post(super::not_implemented))
        .route("/notifications", delete(super::not_implemented).get(super::not_implemented).put(super::not_implemented))
        .route("/notifications/{id}", delete(super::not_implemented).get(super::not_implemented).put(super::not_implemented))
}
