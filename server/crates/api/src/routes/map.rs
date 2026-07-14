//! Immich-compatible routes: map markers and reverse geocoding.

use crate::dto::MapMarkerResponseDto;
use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/map/markers", get(get_map_markers))
        .route("/map/reverse-geocode", get(reverse_geocode))
}

async fn get_map_markers(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<Vec<MapMarkerResponseDto>>> {
    let markers = state
        .services
        .asset
        .map_markers(ctx.user_id)
        .await?
        .iter()
        .filter_map(|(asset, exif)| MapMarkerResponseDto::from_asset_exif(asset, exif))
        .collect();
    Ok(Json(markers))
}

#[derive(Deserialize)]
struct ReverseGeocodeQuery {
    lat: f64,
    lon: f64,
}

async fn reverse_geocode(
    Auth(_): Auth,
    Query(query): Query<ReverseGeocodeQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "lat": query.lat,
        "lon": query.lon,
        "city": null,
        "state": null,
        "country": null,
    })))
}
