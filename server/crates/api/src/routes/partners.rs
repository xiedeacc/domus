//! Immich-compatible routes: partner sharing.

use crate::dto::PartnerResponseDto;
use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/partners", get(list_partners).post(create_partner))
        .route("/partners/{id}", delete(remove_partner).post(create_partner_deprecated).put(update_partner))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePartnerDto {
    shared_with_id: Uuid,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePartnerDto {
    in_timeline: bool,
}

async fn list_partners(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<Vec<PartnerResponseDto>>> {
    let partners = state.services.partner.list(ctx.user_id).await?;
    Ok(Json(partners.iter().map(Into::into).collect()))
}

async fn create_partner(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<CreatePartnerDto>,
) -> ApiResult<Json<PartnerResponseDto>> {
    let partner = state
        .services
        .partner
        .create(ctx.user_id, dto.shared_with_id)
        .await?;
    Ok(Json((&partner).into()))
}

async fn create_partner_deprecated(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<PartnerResponseDto>> {
    let partner = state.services.partner.create(ctx.user_id, id).await?;
    Ok(Json((&partner).into()))
}

async fn update_partner(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdatePartnerDto>,
) -> ApiResult<Json<PartnerResponseDto>> {
    let partner = state
        .services
        .partner
        .update_timeline(ctx.user_id, id, dto.in_timeline)
        .await?;
    Ok(Json((&partner).into()))
}

async fn remove_partner(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.services.partner.remove(ctx.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
