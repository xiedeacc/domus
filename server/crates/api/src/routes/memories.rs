//! Immich-compatible routes: "on this day" memories.

use crate::dto::{AssetResponseDto, MemoryResponseDto};
use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use uuid::Uuid;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/memories", get(list_memories).post(list_memories))
        .route("/memories/statistics", get(statistics))
        .route("/memories/{id}", delete(super::not_implemented).get(get_memory).put(super::not_implemented))
        .route("/memories/{id}/assets", delete(super::not_implemented).put(super::not_implemented))
}

async fn list_memories(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<Vec<MemoryResponseDto>>> {
    let memories = state.services.memory.list(ctx.user_id).await?;
    let mut response = Vec::with_capacity(memories.len());
    for memory in memories {
        let assets = state.services.memory.assets(memory.id).await?;
        response.push(MemoryResponseDto::from_memory(
            &memory,
            assets
                .iter()
                .map(|asset| AssetResponseDto::from_asset(asset, None))
                .collect(),
        ));
    }
    Ok(Json(response))
}

async fn get_memory(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<MemoryResponseDto>> {
    let memory = state.services.memory.get(id).await?;
    let assets = state.services.memory.assets(id).await?;
    Ok(Json(MemoryResponseDto::from_memory(
        &memory,
        assets
            .iter()
            .map(|asset| AssetResponseDto::from_asset(asset, None))
            .collect(),
    )))
}

async fn statistics(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.services.memory.statistics(ctx.user_id).await?))
}
