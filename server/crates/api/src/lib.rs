//! HTTP layer: Immich-compatible REST API under `/api`, socket.io realtime
//! channel under `/api/socket.io`, and static serving of the Flutter web
//! build for everything else.

pub mod dto;
pub mod error;
pub mod extractors;
pub mod routes;
pub mod state;
pub mod websocket;

use axum::routing::get;
use axum::{Json, Router};
use state::AppState;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Assemble the full application router.
pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .merge(routes::activities::router())
        .merge(routes::albums::router())
        .merge(routes::api_keys::router())
        .merge(routes::assets::router())
        .merge(routes::auth::router())
        .merge(routes::download::router())
        .merge(routes::duplicates::router())
        .merge(routes::faces::router())
        .merge(routes::immich_derivatives::router())
        .merge(routes::jobs::router())
        .merge(routes::libraries::router())
        .merge(routes::map::router())
        .merge(routes::memories::router())
        .merge(routes::notifications::router())
        .merge(routes::oauth::router())
        .merge(routes::partners::router())
        .merge(routes::people::router())
        .merge(routes::queues::router())
        .merge(routes::search::router())
        .merge(routes::server::router())
        .merge(routes::sessions::router())
        .merge(routes::shared_links::router())
        .merge(routes::stacks::router())
        .merge(routes::sync::router())
        .merge(routes::system_config::router())
        .merge(routes::system_metadata::router())
        .merge(routes::tags::router())
        .merge(routes::timeline::router())
        .merge(routes::trash::router())
        .merge(routes::users::router())
        .merge(routes::views::router());

    Router::new()
        .route("/.well-known/immich", get(well_known_immich))
        .nest("/api", api.clone())
        .merge(api)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn well_known_immich() -> Json<serde_json::Value> {
    Json(well_known_immich_value())
}

fn well_known_immich_value() -> serde_json::Value {
    serde_json::json!({
        "api": {
            "endpoint": "/api"
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_points_native_clients_to_api_endpoint() {
        assert_eq!(
            well_known_immich_value(),
            serde_json::json!({ "api": { "endpoint": "/api" } })
        );
    }
}
