//! Immich-compatible routes: asset stacks.

use crate::dto::{AssetResponseDto, StackResponseDto};
use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::delete;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stacks", delete(super::not_implemented).get(list_stacks).post(create_stack))
        .route("/stacks/{id}", delete(delete_stack).get(get_stack).put(super::not_implemented))
        .route("/stacks/{id}/assets/{assetId}", delete(remove_asset_from_stack))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateStackDto {
    asset_ids: Vec<Uuid>,
}

async fn list_stacks(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<Vec<StackResponseDto>>> {
    let stacks = state.services.stack.list(ctx.user_id).await?;
    let mut response = Vec::with_capacity(stacks.len());
    for stack in stacks {
        let assets = state.services.stack.assets(stack.id).await?;
        response.push(StackResponseDto::from_stack(
            &stack,
            assets
                .iter()
                .map(|asset| AssetResponseDto::from_asset(asset, None))
                .collect(),
        ));
    }
    Ok(Json(response))
}

async fn create_stack(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<CreateStackDto>,
) -> ApiResult<Json<StackResponseDto>> {
    let stack = state
        .services
        .stack
        .create(ctx.user_id, &dto.asset_ids)
        .await?;
    let assets = state.services.stack.assets(stack.id).await?;
    Ok(Json(StackResponseDto::from_stack(
        &stack,
        assets
            .iter()
            .map(|asset| AssetResponseDto::from_asset(asset, None))
            .collect(),
    )))
}

async fn get_stack(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<StackResponseDto>> {
    let stack = state.services.stack.get(id).await?;
    let assets = state.services.stack.assets(id).await?;
    Ok(Json(StackResponseDto::from_stack(
        &stack,
        assets
            .iter()
            .map(|asset| AssetResponseDto::from_asset(asset, None))
            .collect(),
    )))
}

async fn delete_stack(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.services.stack.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_asset_from_stack(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path((id, asset_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    state.services.stack.remove_asset(id, asset_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
