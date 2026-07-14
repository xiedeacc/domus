//! Immich-compatible routes: trash restore/empty.

use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trash/empty", post(empty_trash))
        .route("/trash/restore", post(restore_all))
        .route("/trash/restore/assets", post(restore_assets))
}

async fn empty_trash(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<serde_json::Value>> {
    let count = state.services.trash.empty(ctx.user_id).await?;
    Ok(Json(trash_response(count)))
}

async fn restore_all(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<serde_json::Value>> {
    let count = state.services.trash.restore_all(ctx.user_id).await?;
    Ok(Json(trash_response(count)))
}

#[derive(Deserialize)]
struct RestoreAssetsDto {
    ids: Vec<Uuid>,
}

async fn restore_assets(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Json(dto): Json<RestoreAssetsDto>,
) -> ApiResult<Json<serde_json::Value>> {
    let count = state.services.trash.restore(&dto.ids).await?;
    Ok(Json(trash_response(count)))
}

fn trash_response(count: u64) -> serde_json::Value {
    serde_json::json!({ "count": count })
}

#[cfg(test)]
mod tests {
    use super::trash_response;

    #[test]
    fn trash_response_matches_immich_shape() {
        assert_eq!(trash_response(0), serde_json::json!({ "count": 0 }));
        assert_eq!(trash_response(2), serde_json::json!({ "count": 2 }));
    }
}
