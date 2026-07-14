//! Immich-compatible routes: map markers and reverse geocoding.

use crate::dto::MapMarkerResponseDto;
use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use domus_common::Error;
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
    validate_lat_lon(query.lat, query.lon)?;
    Ok(Json(serde_json::json!([{
        "city": null,
        "state": null,
        "country": null,
    }])))
}

fn validate_lat_lon(lat: f64, lon: f64) -> Result<(), Error> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(Error::BadRequest("lat must be between -90 and 90".into()));
    }
    if !(-180.0..=180.0).contains(&lon) {
        return Err(Error::BadRequest("lon must be between -180 and 180".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_lat_lon;

    #[test]
    fn validate_reverse_geocode_coordinates_like_immich() {
        assert!(validate_lat_lon(42.0, 69.0).is_ok());
        assert!(validate_lat_lon(-90.0, -180.0).is_ok());
        assert!(validate_lat_lon(90.0, 180.0).is_ok());
        assert!(validate_lat_lon(91.0, 0.0).is_err());
        assert!(validate_lat_lon(0.0, 181.0).is_err());
    }
}
