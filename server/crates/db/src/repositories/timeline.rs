use crate::PgPool;
use base64::Engine;
use chrono::NaiveDate;
use domus_common::{Error, Result};
use uuid::Uuid;

/// Filter set shared by /timeline/buckets and /timeline/bucket.
#[derive(Debug, Clone, Default)]
pub struct TimeBucketQuery {
    pub user_ids: Vec<Uuid>,
    pub album_id: Option<Uuid>,
    pub person_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub bbox: Option<BBox>,
    pub is_favorite: Option<bool>,
    pub is_trashed: Option<bool>,
    pub visibility: Option<String>,
    pub with_partners: bool,
    pub with_stacked: bool,
    pub order_desc: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

#[derive(Clone)]
pub struct TimelineRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl TimelineRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_time_buckets(&self, query: TimeBucketQuery) -> Result<Vec<(String, i64)>> {
        if query.person_id.is_some() {
            return Ok(vec![]);
        }
        let owner_id = query.user_ids.first().copied().unwrap_or_default();
        let trashed = query.is_trashed.unwrap_or(false);
        let order = if query.order_desc { "DESC" } else { "ASC" };
        let sql = format!(
            r#"SELECT substr(a."localDateTime", 1, 7) || '-01' AS bucket,
                      COUNT(*) AS count
               FROM asset a
               WHERE a."ownerId" = $1
                 AND ($2 IS NULL OR EXISTS (
                     SELECT 1 FROM album_asset aa WHERE aa."assetId" = a.id AND aa."albumId" = $2
                 ))
                 AND ($3 IS NULL OR EXISTS (
                     SELECT 1 FROM tag_asset ta WHERE ta."assetId" = a.id AND ta."tagId" = $3
                 ))
                 AND ($4 IS NULL OR a."isFavorite" = $4)
                 AND (($5 = 0 AND a."deletedAt" IS NULL) OR ($5 = 1 AND a."deletedAt" IS NOT NULL))
                 AND (($6 IS NOT NULL AND a.visibility = $6) OR ($6 IS NULL AND a.visibility = 'timeline'))
               GROUP BY bucket
               ORDER BY bucket {order}"#,
        );
        sqlx::query_as(&sql)
            .bind(owner_id)
            .bind(query.album_id)
            .bind(query.tag_id)
            .bind(query.is_favorite)
            .bind(trashed)
            .bind(query.visibility.as_deref())
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)
    }

    pub async fn get_time_bucket_assets(
        &self,
        bucket: &str,
        query: TimeBucketQuery,
    ) -> Result<serde_json::Value> {
        if query.person_id.is_some() {
            return Ok(empty_bucket(bucket));
        }
        let _bucket_date = NaiveDate::parse_from_str(bucket, "%Y-%m-%d")
            .map_err(|_| Error::BadRequest("Invalid time bucket format".into()))?;
        let owner_id = query.user_ids.first().copied().unwrap_or_default();
        let trashed = query.is_trashed.unwrap_or(false);
        let order = if query.order_desc { "DESC" } else { "ASC" };
        let sql = format!(
            r#"SELECT a.id, a."ownerId" AS owner_id, a.type AS asset_type, a."isFavorite" AS is_favorite,
                      a.visibility, a."deletedAt" AS deleted_at, a."createdAt" AS created_at,
                      a."fileCreatedAt" AS file_created_at, CAST(a.duration AS TEXT) AS duration,
                      a."livePhotoVideoId" AS live_photo_video_id, a.thumbhash,
                      COALESCE(e."exifImageWidth", 1) AS width,
                      COALESCE(NULLIF(e."exifImageHeight", 0), 1) AS height,
                      e."projectionType" AS projection_type,
                      e.city, e.country, e.latitude, e.longitude
               FROM asset a
               LEFT JOIN asset_exif e ON e."assetId" = a.id
               WHERE a."ownerId" = $1
                 AND substr(a."localDateTime", 1, 7) || '-01' = $2
                 AND ($3 IS NULL OR EXISTS (
                     SELECT 1 FROM album_asset aa WHERE aa."assetId" = a.id AND aa."albumId" = $3
                 ))
                 AND ($4 IS NULL OR EXISTS (
                     SELECT 1 FROM tag_asset ta WHERE ta."assetId" = a.id AND ta."tagId" = $4
                 ))
                 AND ($5 IS NULL OR a."isFavorite" = $5)
                 AND (($6 = 0 AND a."deletedAt" IS NULL) OR ($6 = 1 AND a."deletedAt" IS NOT NULL))
                 AND (($7 IS NOT NULL AND a.visibility = $7) OR ($7 IS NULL AND a.visibility = 'timeline'))
               ORDER BY a."localDateTime" {order}, a.id"#,
        );
        let rows = sqlx::query_as::<_, TimelineAssetRow>(&sql)
            .bind(owner_id)
            .bind(bucket)
            .bind(query.album_id)
            .bind(query.tag_id)
            .bind(query.is_favorite)
            .bind(trashed)
            .bind(query.visibility.as_deref())
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(rows_to_bucket(bucket, rows))
    }
}

#[derive(sqlx::FromRow)]
struct TimelineAssetRow {
    id: Uuid,
    owner_id: Uuid,
    asset_type: String,
    is_favorite: bool,
    visibility: String,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    file_created_at: chrono::DateTime<chrono::Utc>,
    duration: Option<String>,
    live_photo_video_id: Option<Uuid>,
    thumbhash: Option<Vec<u8>>,
    width: i32,
    height: i32,
    projection_type: Option<String>,
    city: Option<String>,
    country: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

fn rows_to_bucket(bucket: &str, rows: Vec<TimelineAssetRow>) -> serde_json::Value {
    let mut id = Vec::with_capacity(rows.len());
    let mut owner_id = Vec::with_capacity(rows.len());
    let mut ratio = Vec::with_capacity(rows.len());
    let mut is_favorite = Vec::with_capacity(rows.len());
    let mut visibility = Vec::with_capacity(rows.len());
    let mut is_trashed = Vec::with_capacity(rows.len());
    let mut is_image = Vec::with_capacity(rows.len());
    let mut thumbhash: Vec<Option<String>> = Vec::with_capacity(rows.len());
    let mut created_at = Vec::with_capacity(rows.len());
    let mut file_created_at = Vec::with_capacity(rows.len());
    let mut local_offset_hours = Vec::with_capacity(rows.len());
    let mut duration = Vec::with_capacity(rows.len());
    let mut projection_type = Vec::with_capacity(rows.len());
    let mut live_photo_video_id = Vec::with_capacity(rows.len());
    let mut city = Vec::with_capacity(rows.len());
    let mut country = Vec::with_capacity(rows.len());
    let mut latitude = Vec::with_capacity(rows.len());
    let mut longitude = Vec::with_capacity(rows.len());

    for row in rows {
        id.push(row.id);
        owner_id.push(row.owner_id);
        ratio.push(row.width as f64 / row.height.max(1) as f64);
        is_favorite.push(row.is_favorite);
        visibility.push(row.visibility);
        is_trashed.push(row.deleted_at.is_some());
        is_image.push(row.asset_type == "IMAGE");
        thumbhash.push(
            row.thumbhash
                .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
        );
        created_at.push(iso(row.created_at));
        file_created_at.push(iso(row.file_created_at));
        local_offset_hours.push(0.0);
        duration.push(duration_millis(row.duration.as_deref()));
        projection_type.push(row.projection_type);
        live_photo_video_id.push(row.live_photo_video_id);
        city.push(row.city);
        country.push(row.country);
        latitude.push(row.latitude);
        longitude.push(row.longitude);
    }

    serde_json::json!({
        "id": id,
        "ownerId": owner_id,
        "ratio": ratio,
        "isFavorite": is_favorite,
        "visibility": visibility,
        "isTrashed": is_trashed,
        "isImage": is_image,
        "thumbhash": thumbhash,
        "createdAt": created_at,
        "fileCreatedAt": file_created_at,
        "localOffsetHours": local_offset_hours,
        "duration": duration,
        "projectionType": projection_type,
        "livePhotoVideoId": live_photo_video_id,
        "city": city,
        "country": country,
        "latitude": latitude,
        "longitude": longitude,
        "timeBucket": bucket,
        "count": id.len(),
    })
}

fn empty_bucket(bucket: &str) -> serde_json::Value {
    rows_to_bucket(bucket, vec![])
}

fn iso(dt: chrono::DateTime<chrono::Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn duration_millis(value: Option<&str>) -> Option<i64> {
    let value = value?;
    if let Ok(millis) = value.parse::<i64>() {
        return Some(millis);
    }
    let mut parts = value.split(':');
    let hours = parts.next()?.parse::<i64>().ok()?;
    let minutes = parts.next()?.parse::<i64>().ok()?;
    let seconds = parts.next()?.parse::<f64>().ok()?;
    Some(((hours * 3600 + minutes * 60) as f64 * 1000.0 + seconds * 1000.0).round() as i64)
}

fn db_err(e: sqlx::Error) -> Error {
    match e {
        sqlx::Error::RowNotFound => Error::NotFound("row not found".into()),
        other => Error::Database(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_millis_converts_immich_duration_strings() {
        assert_eq!(duration_millis(Some("00:00:01.500")), Some(1500));
        assert_eq!(duration_millis(Some("01:02:03.004")), Some(3_723_004));
        assert_eq!(duration_millis(Some("2501")), Some(2501));
        assert_eq!(duration_millis(None), None);
        assert_eq!(duration_millis(Some("bad")), None);
    }

    #[test]
    fn empty_bucket_uses_columnar_shape() {
        let value = empty_bucket("2026-07-01");
        assert_eq!(value["timeBucket"], "2026-07-01");
        assert_eq!(value["count"], 0);
        assert!(value["id"].as_array().unwrap().is_empty());
        assert!(value["ownerId"].as_array().unwrap().is_empty());
        assert!(value["fileCreatedAt"].as_array().unwrap().is_empty());
    }
}
