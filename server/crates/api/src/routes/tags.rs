//! Immich-compatible routes: hierarchical tags.

use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use chrono::{DateTime, SecondsFormat, Utc};
use domus_db::entities::Tag;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tags", get(list_tags).post(create_tag).put(upsert_tags))
        .route("/tags/assets", put(tag_assets_bulk))
        .route("/tags/{id}", delete(delete_tag).get(get_tag).put(update_tag))
        .route("/tags/{id}/assets", delete(untag_assets).put(tag_assets))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TagResponseDto {
    id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<Uuid>,
    name: String,
    value: String,
    created_at: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

impl From<&Tag> for TagResponseDto {
    fn from(tag: &Tag) -> Self {
        Self {
            id: tag.id,
            parent_id: tag.parent_id,
            name: tag
                .value
                .rsplit('/')
                .next()
                .unwrap_or(&tag.value)
                .to_owned(),
            value: tag.value.clone(),
            created_at: iso(&tag.created_at),
            updated_at: iso(&tag.updated_at),
            color: tag.color.clone(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTagDto {
    name: String,
    parent_id: Option<Uuid>,
    color: Option<String>,
}

async fn list_tags(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<Vec<TagResponseDto>>> {
    let tags = state.services.tag.list(ctx.user_id).await?;
    Ok(Json(tags.iter().map(Into::into).collect()))
}

async fn create_tag(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<CreateTagDto>,
) -> ApiResult<Json<TagResponseDto>> {
    let tag = state
        .services
        .tag
        .create(ctx.user_id, &dto.name, dto.parent_id, dto.color.as_deref())
        .await?;
    Ok(Json((&tag).into()))
}

#[derive(Deserialize)]
struct UpsertTagsDto {
    tags: Vec<String>,
}

async fn upsert_tags(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<UpsertTagsDto>,
) -> ApiResult<Json<Vec<TagResponseDto>>> {
    let mut tags = Vec::with_capacity(dto.tags.len());
    for value in dto.tags {
        tags.push(state.services.tag.upsert(ctx.user_id, &value).await?);
    }
    Ok(Json(tags.iter().map(Into::into).collect()))
}

async fn get_tag(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TagResponseDto>> {
    let tag = state.services.tag.get(id).await?;
    Ok(Json((&tag).into()))
}

#[derive(Deserialize)]
struct UpdateTagDto {
    color: Option<String>,
}

async fn update_tag(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateTagDto>,
) -> ApiResult<Json<TagResponseDto>> {
    let tag = state
        .services
        .tag
        .update_color(id, dto.color.as_deref())
        .await?;
    Ok(Json((&tag).into()))
}

async fn delete_tag(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.services.tag.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TagAssetsDto {
    asset_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BulkTagAssetsDto {
    tag_ids: Vec<Uuid>,
    asset_ids: Vec<Uuid>,
}

async fn tag_assets(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<TagAssetsDto>,
) -> ApiResult<Json<serde_json::Value>> {
    let count = state.services.tag.tag_assets(id, &dto.asset_ids).await?;
    Ok(Json(serde_json::json!({ "count": count })))
}

async fn untag_assets(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Path(id): Path<Uuid>,
    Json(dto): Json<TagAssetsDto>,
) -> ApiResult<Json<serde_json::Value>> {
    let count = state.services.tag.untag_assets(id, &dto.asset_ids).await?;
    Ok(Json(serde_json::json!({ "count": count })))
}

async fn tag_assets_bulk(
    State(state): State<AppState>,
    Auth(_ctx): Auth,
    Json(dto): Json<BulkTagAssetsDto>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut count = 0;
    for tag_id in dto.tag_ids {
        count += state
            .services
            .tag
            .tag_assets(tag_id, &dto.asset_ids)
            .await?;
    }
    Ok(Json(serde_json::json!({ "count": count })))
}

fn iso(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn tag_response_maps_immich_shape_and_leaf_name() {
        let tag = tag("Parent/Child", Some("#000000"), Some(Uuid::nil()));

        let value = serde_json::to_value(TagResponseDto::from(&tag)).unwrap();

        assert_eq!(value["id"], tag.id.to_string());
        assert_eq!(value["parentId"], Uuid::nil().to_string());
        assert_eq!(value["name"], "Child");
        assert_eq!(value["value"], "Parent/Child");
        assert_eq!(value["createdAt"], "2026-07-14T12:13:14.000Z");
        assert_eq!(value["updatedAt"], "2026-07-14T12:13:14.000Z");
        assert_eq!(value["color"], "#000000");
    }

    #[test]
    fn tag_response_omits_optional_null_fields_like_immich() {
        let tag = tag("Parent", None, None);

        let value = serde_json::to_value(TagResponseDto::from(&tag)).unwrap();
        let object = value.as_object().unwrap();

        assert!(!object.contains_key("parentId"));
        assert!(!object.contains_key("color"));
    }

    fn tag(value: &str, color: Option<&str>, parent_id: Option<Uuid>) -> Tag {
        let dt = Utc.with_ymd_and_hms(2026, 7, 14, 12, 13, 14).unwrap();
        Tag {
            id: Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap(),
            user_id: Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap(),
            value: value.to_owned(),
            color: color.map(str::to_owned),
            parent_id,
            created_at: dt,
            updated_at: dt,
        }
    }
}
