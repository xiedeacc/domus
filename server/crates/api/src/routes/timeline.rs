//! /timeline — month buckets + columnar bucket contents.

use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use domus_common::{Error, Result};
use domus_db::repositories::timeline::{BBox, TimeBucketQuery};
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/timeline/buckets", get(get_time_buckets))
        .route("/timeline/bucket", get(get_time_bucket))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimelineParams {
    album_id: Option<Uuid>,
    person_id: Option<Uuid>,
    tag_id: Option<Uuid>,
    user_id: Option<Uuid>,
    is_favorite: Option<bool>,
    is_trashed: Option<bool>,
    visibility: Option<String>,
    with_partners: Option<bool>,
    with_stacked: Option<bool>,
    order: Option<String>,
    time_bucket: Option<String>,
    bbox: Option<String>,
    // shared-link auth (resolved by the Auth extractor once implemented)
    #[allow(dead_code)]
    key: Option<String>,
    #[allow(dead_code)]
    slug: Option<String>,
}

impl TimelineParams {
    fn to_query(&self, user_id: Uuid) -> Result<TimeBucketQuery> {
        Ok(TimeBucketQuery {
            user_ids: vec![self.user_id.unwrap_or(user_id)],
            album_id: self.album_id,
            person_id: self.person_id,
            tag_id: self.tag_id,
            bbox: self.bbox.as_deref().map(parse_bbox).transpose()?,
            is_favorite: self.is_favorite,
            is_trashed: self.is_trashed,
            visibility: self.visibility.clone(),
            with_partners: self.with_partners.unwrap_or(false),
            with_stacked: self.with_stacked.unwrap_or(false),
            order_desc: self.order.as_deref() != Some("asc"),
        })
    }
}

async fn get_time_buckets(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Query(params): Query<TimelineParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let buckets = state
        .services
        .timeline
        .buckets(params.to_query(ctx.user_id)?)
        .await?;
    Ok(Json(serde_json::to_value(buckets).unwrap()))
}

async fn get_time_bucket(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Query(params): Query<TimelineParams>,
) -> ApiResult<Json<serde_json::Value>> {
    let bucket = params
        .time_bucket
        .clone()
        .ok_or_else(|| domus_common::Error::BadRequest("timeBucket is required".into()))?;
    let assets = state
        .services
        .timeline
        .bucket_assets(&bucket, params.to_query(ctx.user_id)?)
        .await?;
    Ok(Json(assets))
}

fn parse_bbox(value: &str) -> Result<BBox> {
    let parts: Vec<_> = value.split(',').collect();
    if parts.len() != 4 {
        return Err(Error::BadRequest(
            "bbox must have 4 comma-separated numbers: west,south,east,north".into(),
        ));
    }
    let numbers: std::result::Result<Vec<f64>, _> =
        parts.into_iter().map(str::parse::<f64>).collect();
    let numbers =
        numbers.map_err(|_| Error::BadRequest("bbox parts must be valid numbers".into()))?;
    Ok(BBox {
        west: numbers[0],
        south: numbers[1],
        east: numbers[2],
        north: numbers[3],
    })
}

#[cfg(test)]
mod tests {
    use super::parse_bbox;

    #[test]
    fn parse_bbox_matches_immich_query_transform() {
        let bbox = parse_bbox("11.075683,49.416711,11.117589,49.454875").unwrap();
        assert_eq!(bbox.west, 11.075683);
        assert_eq!(bbox.south, 49.416711);
        assert_eq!(bbox.east, 11.117589);
        assert_eq!(bbox.north, 49.454875);
    }

    #[test]
    fn parse_bbox_rejects_incomplete_or_invalid_values() {
        assert!(parse_bbox("1,2,3").is_err());
        assert!(parse_bbox("1,2,3,invalid").is_err());
    }
}
