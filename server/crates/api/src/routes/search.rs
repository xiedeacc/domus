//! Immich-compatible routes: metadata/people/places search; smart search 501s without ML.

use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search/cities", get(cities))
        .route("/search/explore", get(explore))
        .route("/search/large-assets", post(super::not_implemented))
        .route("/search/metadata", post(metadata))
        .route("/search/person", get(super::not_implemented))
        .route("/search/places", get(places))
        .route("/search/random", post(random))
        .route("/search/smart", post(super::not_implemented))
        .route("/search/statistics", post(super::not_implemented))
        .route("/search/suggestions", get(suggestions))
}

async fn metadata(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(filters): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(
        state
            .services
            .search
            .search_metadata(ctx.user_id, filters)
            .await?,
    ))
}

async fn random(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(
        state
            .services
            .search
            .search_metadata(ctx.user_id, serde_json::json!({}))
            .await?,
    ))
}

async fn explore(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.services.search.explore(ctx.user_id).await?))
}

#[derive(Deserialize)]
struct SuggestionQuery {
    #[serde(default = "default_suggestion_kind")]
    kind: String,
}

async fn suggestions(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Query(query): Query<SuggestionQuery>,
) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(
        state
            .services
            .search
            .suggestions(ctx.user_id, &query.kind)
            .await?,
    ))
}

async fn cities(State(state): State<AppState>, Auth(ctx): Auth) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(
        state
            .services
            .search
            .suggestions(ctx.user_id, "city")
            .await?,
    ))
}

async fn places(State(state): State<AppState>, Auth(ctx): Auth) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(
        state
            .services
            .search
            .suggestions(ctx.user_id, "country")
            .await?,
    ))
}

fn default_suggestion_kind() -> String {
    "city".to_owned()
}
