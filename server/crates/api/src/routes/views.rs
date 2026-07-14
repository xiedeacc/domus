//! Immich-compatible routes: folder view of original paths.

use crate::dto::AssetResponseDto;
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
        .route("/view/folder", get(folder_assets))
        .route("/view/folder/unique-paths", get(unique_paths))
}

#[derive(Deserialize)]
struct FolderQuery {
    path: String,
}

async fn unique_paths(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(
        state
            .services
            .asset
            .unique_original_folders(ctx.user_id)
            .await?,
    ))
}

async fn folder_assets(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Query(query): Query<FolderQuery>,
) -> ApiResult<Json<Vec<AssetResponseDto>>> {
    let assets = state
        .services
        .asset
        .assets_by_original_folder(ctx.user_id, &query.path)
        .await?;
    let mut response = Vec::with_capacity(assets.len());
    for asset in assets {
        let exif = state.services.asset.get(asset.id).await?.1;
        response.push(AssetResponseDto::from_asset(&asset, exif.as_ref()));
    }
    Ok(Json(response))
}
