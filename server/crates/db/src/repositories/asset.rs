use super::db_err;
use crate::entities::{Asset, Exif};
use crate::PgPool;
use domus_common::types::{AssetType, AssetVisibility};
use domus_common::{Error, Result};
use sqlx::Row;
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Clone)]
pub struct AssetRepository {
    pool: PgPool,
}

pub struct CreateAsset {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub device_asset_id: String,
    pub device_id: String,
    pub asset_type: domus_common::types::AssetType,
    pub original_path: String,
    pub original_file_name: String,
    pub checksum: Vec<u8>,
    pub file_created_at: chrono::DateTime<chrono::Utc>,
    pub file_modified_at: chrono::DateTime<chrono::Utc>,
    pub is_favorite: bool,
    pub live_photo_video_id: Option<Uuid>,
    pub visibility: domus_common::types::AssetVisibility,
}

impl AssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: Uuid) -> Result<Asset> {
        let row = sqlx::query(ASSET_SELECT_SQL)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(db_err)?;
        asset_from_row(&row)
    }

    pub async fn list_by_album(&self, album_id: Uuid) -> Result<Vec<Asset>> {
        let rows = sqlx::query(
            r#"SELECT a.id, a."ownerId", a."libraryId", lower(hex(a.id)) AS "deviceAssetId", 'domus' AS "deviceId", a.type,
                      a."originalPath", a."originalFileName", a.checksum, a.visibility,
                      a."isFavorite", a."isOffline", a."isExternal", a."livePhotoVideoId",
                      a."stackId", CAST(a.duration AS TEXT) AS duration, a.thumbhash, a."fileCreatedAt", a."fileModifiedAt",
                      a."localDateTime", a."createdAt", a."updatedAt", a."deletedAt"
               FROM asset a
               JOIN album_asset aa ON aa."assetId" = a.id
               WHERE aa."albumId" = $1 AND a."deletedAt" IS NULL
               ORDER BY aa."createdAt" DESC"#,
        )
        .bind(album_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        rows.iter().map(asset_from_row).collect()
    }

    pub async fn list_by_ids(&self, owner_id: Uuid, ids: &[Uuid]) -> Result<Vec<Asset>> {
        let mut assets = Vec::new();
        for id in ids {
            let row = sqlx::query(ASSET_SELECT_SQL)
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(db_err)?;
            if let Some(row) = row {
                let asset = asset_from_row(&row)?;
                if asset.owner_id == owner_id && asset.deleted_at.is_none() {
                    assets.push(asset);
                }
            }
        }
        assets.sort_by(|a, b| b.local_date_time.cmp(&a.local_date_time));
        Ok(assets)
    }

    pub async fn list_by_stack(&self, stack_id: Uuid) -> Result<Vec<Asset>> {
        let rows = sqlx::query(
            r#"SELECT id, "ownerId", "libraryId", lower(hex(id)) AS "deviceAssetId", 'domus' AS "deviceId", type,
                      "originalPath", "originalFileName", checksum, visibility,
                      "isFavorite", "isOffline", "isExternal", "livePhotoVideoId",
                      "stackId", CAST(duration AS TEXT) AS duration, thumbhash, "fileCreatedAt", "fileModifiedAt",
                      "localDateTime", "createdAt", "updatedAt", "deletedAt"
               FROM asset
               WHERE "stackId" = $1 AND "deletedAt" IS NULL
               ORDER BY "localDateTime" DESC"#,
        )
        .bind(stack_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        rows.iter().map(asset_from_row).collect()
    }

    pub async fn map_markers(&self, owner_id: Uuid) -> Result<Vec<(Asset, Exif)>> {
        let rows = sqlx::query(
            r#"SELECT a.id, a."ownerId", a."libraryId", lower(hex(a.id)) AS "deviceAssetId", 'domus' AS "deviceId", a.type,
                      a."originalPath", a."originalFileName", a.checksum, a.visibility,
                      a."isFavorite", a."isOffline", a."isExternal", a."livePhotoVideoId",
                      a."stackId", CAST(a.duration AS TEXT) AS duration, a.thumbhash, a."fileCreatedAt", a."fileModifiedAt",
                      a."localDateTime", a."createdAt", a."updatedAt", a."deletedAt",
                      e."assetId" AS exif_asset_id, e.make, e.model,
                      e."exifImageWidth", e."exifImageHeight", e."fileSizeInByte",
                      e.orientation, e."dateTimeOriginal", e."modifyDate", e."timeZone",
                      e.latitude, e.longitude, e.city, e.state, e.country, e.description,
                      e."fNumber", e."focalLength", e.iso, e."exposureTime",
                      e."lensModel", e."projectionType", e.rating, e.fps
               FROM asset a
               JOIN asset_exif e ON e."assetId" = a.id
               WHERE a."ownerId" = $1
                 AND a."deletedAt" IS NULL
                 AND e.latitude IS NOT NULL
                 AND e.longitude IS NOT NULL
               ORDER BY a."localDateTime" DESC"#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;

        rows.iter()
            .map(|row| {
                Ok((
                    asset_from_row(row)?,
                    Exif {
                        asset_id: row.try_get("exif_asset_id").map_err(db_err)?,
                        make: row.try_get("make").map_err(db_err)?,
                        model: row.try_get("model").map_err(db_err)?,
                        exif_image_width: row.try_get("exifImageWidth").map_err(db_err)?,
                        exif_image_height: row.try_get("exifImageHeight").map_err(db_err)?,
                        file_size_in_byte: row.try_get("fileSizeInByte").map_err(db_err)?,
                        orientation: row.try_get("orientation").map_err(db_err)?,
                        date_time_original: row.try_get("dateTimeOriginal").map_err(db_err)?,
                        modify_date: row.try_get("modifyDate").map_err(db_err)?,
                        time_zone: row.try_get("timeZone").map_err(db_err)?,
                        latitude: row.try_get("latitude").map_err(db_err)?,
                        longitude: row.try_get("longitude").map_err(db_err)?,
                        city: row.try_get("city").map_err(db_err)?,
                        state: row.try_get("state").map_err(db_err)?,
                        country: row.try_get("country").map_err(db_err)?,
                        description: row.try_get("description").map_err(db_err)?,
                        f_number: row.try_get("fNumber").map_err(db_err)?,
                        focal_length: row.try_get("focalLength").map_err(db_err)?,
                        iso: row.try_get("iso").map_err(db_err)?,
                        exposure_time: row.try_get("exposureTime").map_err(db_err)?,
                        lens_model: row.try_get("lensModel").map_err(db_err)?,
                        projection_type: row.try_get("projectionType").map_err(db_err)?,
                        rating: row.try_get("rating").map_err(db_err)?,
                        fps: row.try_get("fps").map_err(db_err)?,
                    },
                ))
            })
            .collect()
    }

    pub async fn unique_original_folders(&self, owner_id: Uuid) -> Result<Vec<String>> {
        let paths: Vec<String> = sqlx::query_scalar(
            r#"SELECT "originalPath"
               FROM asset
               WHERE "ownerId" = $1 AND "deletedAt" IS NULL"#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        let folders = paths
            .iter()
            .filter_map(|path| parent_folder(path))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Ok(folders)
    }

    pub async fn list_by_original_folder(
        &self,
        owner_id: Uuid,
        folder: &str,
    ) -> Result<Vec<Asset>> {
        let rows = sqlx::query(
            r#"SELECT id, "ownerId", "libraryId", lower(hex(id)) AS "deviceAssetId", 'domus' AS "deviceId", type,
                      "originalPath", "originalFileName", checksum, visibility,
                      "isFavorite", "isOffline", "isExternal", "livePhotoVideoId",
                      "stackId", CAST(duration AS TEXT) AS duration, thumbhash, "fileCreatedAt", "fileModifiedAt",
                      "localDateTime", "createdAt", "updatedAt", "deletedAt"
               FROM asset
               WHERE "ownerId" = $1
                 AND "deletedAt" IS NULL
               ORDER BY "localDateTime" DESC"#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        rows.iter()
            .filter_map(|row| match row.try_get::<String, _>("originalPath") {
                Ok(path) if parent_folder(&path).as_deref() == Some(folder) => {
                    Some(asset_from_row(row))
                }
                Ok(_) => None,
                Err(e) => Some(Err(db_err(e))),
            })
            .collect()
    }

    pub async fn get_exif(&self, asset_id: Uuid) -> Result<Option<Exif>> {
        sqlx::query_as::<_, ExifRow>(
            r#"SELECT "assetId" AS asset_id, make, model, "exifImageWidth" AS exif_image_width,
                      "exifImageHeight" AS exif_image_height, "fileSizeInByte" AS file_size_in_byte,
                      orientation, "dateTimeOriginal" AS date_time_original, "modifyDate" AS modify_date,
                      "timeZone" AS time_zone, latitude, longitude, city, state, country, description,
                      "fNumber" AS f_number, "focalLength" AS focal_length, iso, "exposureTime" AS exposure_time,
                      "lensModel" AS lens_model, "projectionType" AS projection_type, rating, fps
               FROM asset_exif WHERE "assetId" = $1"#,
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(db_err)
    }

    /// Duplicate detection on upload: find an asset owned by `owner_id`
    /// with the same SHA-1 checksum.
    pub async fn get_by_checksum(&self, owner_id: Uuid, checksum: &[u8]) -> Result<Option<Uuid>> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            r#"SELECT id FROM asset WHERE "ownerId" = $1 AND checksum = $2 LIMIT 1"#,
        )
        .bind(owner_id)
        .bind(checksum)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(row.map(|r| r.0))
    }

    pub async fn create(&self, asset: CreateAsset) -> Result<Asset> {
        let row = sqlx::query(
            r#"INSERT INTO asset (
                   id, "ownerId", type, "originalPath", "originalFileName", checksum,
                   "checksumAlgorithm", visibility, "isFavorite",
                   "livePhotoVideoId", "fileCreatedAt", "fileModifiedAt", "localDateTime"
               )
               VALUES ($1, $2, $3, $4, $5, $6, 'sha1', $7, $8, $9, $10, $11, $10)
               RETURNING id, "ownerId", "libraryId", lower(hex(id)) AS "deviceAssetId", 'domus' AS "deviceId", type,
                         "originalPath", "originalFileName", checksum, visibility,
                         "isFavorite", "isOffline", "isExternal", "livePhotoVideoId",
                         "stackId", CAST(duration AS TEXT) AS duration, thumbhash, "fileCreatedAt", "fileModifiedAt",
                         "localDateTime", "createdAt", "updatedAt", "deletedAt""#,
        )
        .bind(asset.id)
        .bind(asset.owner_id)
        .bind(asset_type_to_db(asset.asset_type))
        .bind(&asset.original_path)
        .bind(&asset.original_file_name)
        .bind(&asset.checksum)
        .bind(visibility_to_db(asset.visibility))
        .bind(asset.is_favorite)
        .bind(asset.live_photo_video_id)
        .bind(asset.file_created_at)
        .bind(asset.file_modified_at)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        asset_from_row(&row)
    }

    pub async fn update_favorite(&self, ids: &[Uuid], is_favorite: bool) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for id in ids {
            sqlx::query(
                r#"UPDATE asset SET "isFavorite" = $1, "updatedAt" = datetime('now') WHERE id = $2"#,
            )
            .bind(is_favorite)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }

    pub async fn update_visibility(&self, ids: &[Uuid], visibility: AssetVisibility) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for id in ids {
            sqlx::query(
                r#"UPDATE asset SET visibility = $1, "updatedAt" = datetime('now') WHERE id = $2"#,
            )
            .bind(visibility_to_db(visibility))
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }

    /// Soft-delete: move assets to trash (sets deletedAt).
    pub async fn trash(&self, ids: &[Uuid]) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for id in ids {
            sqlx::query(
                r#"UPDATE asset SET "deletedAt" = datetime('now'), "updatedAt" = datetime('now') WHERE id = $1"#,
            )
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }

    pub async fn restore(&self, ids: &[Uuid]) -> Result<u64> {
        let mut changed = 0;
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for id in ids {
            let result = sqlx::query(
                r#"UPDATE asset SET "deletedAt" = NULL, "updatedAt" = datetime('now') WHERE id = $1"#,
            )
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
            changed += result.rows_affected();
        }
        tx.commit().await.map_err(db_err)?;
        Ok(changed)
    }

    pub async fn restore_all_for_user(&self, user_id: Uuid) -> Result<u64> {
        let result = sqlx::query(
            r#"UPDATE asset SET "deletedAt" = NULL, "updatedAt" = datetime('now')
               WHERE "ownerId" = $1 AND "deletedAt" IS NOT NULL"#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(result.rows_affected())
    }

    pub async fn trashed_ids_for_user(&self, user_id: Uuid) -> Result<Vec<Uuid>> {
        sqlx::query_scalar(
            r#"SELECT id FROM asset WHERE "ownerId" = $1 AND "deletedAt" IS NOT NULL"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn statistics(&self, user_id: Uuid) -> Result<(i64, i64)> {
        let (images, videos): (i64, i64) = sqlx::query_as(
            r#"SELECT
                   SUM(CASE WHEN type = 'IMAGE' THEN 1 ELSE 0 END) AS images,
                   SUM(CASE WHEN type = 'VIDEO' THEN 1 ELSE 0 END) AS videos
               FROM asset
               WHERE "ownerId" = $1 AND "deletedAt" IS NULL"#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok((images, videos))
    }

    pub async fn find_existing_device_assets(
        &self,
        owner_id: Uuid,
        device_asset_ids: &[String],
    ) -> Result<Vec<String>> {
        let mut existing = Vec::new();
        for id in device_asset_ids {
            let found: Option<String> = sqlx::query_scalar(
                r#"SELECT id FROM asset
                   WHERE "ownerId" = $1 AND id = $2 AND "deletedAt" IS NULL"#,
            )
            .bind(owner_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
            if let Some(id) = found {
                existing.push(id);
            }
        }
        Ok(existing)
    }

    pub async fn list_device_asset_ids_by_device(
        &self,
        owner_id: Uuid,
        _device_id: &str,
    ) -> Result<Vec<String>> {
        let _ = owner_id;
        Ok(vec![])
    }

    pub async fn upsert_exif(&self, exif: &Exif) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO asset_exif (
                   "assetId", make, model, "exifImageWidth", "exifImageHeight", "fileSizeInByte",
                   orientation, "dateTimeOriginal", "modifyDate", "timeZone", latitude, longitude,
                   city, state, country, description, "fNumber", "focalLength", iso, "exposureTime",
                   "lensModel", "projectionType", rating, fps
               )
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                       COALESCE($16, ''), $17, $18, $19, $20, $21, $22, $23, $24)
               ON CONFLICT ("assetId") DO UPDATE SET
                   make = EXCLUDED.make,
                   model = EXCLUDED.model,
                   "exifImageWidth" = EXCLUDED."exifImageWidth",
                   "exifImageHeight" = EXCLUDED."exifImageHeight",
                   "fileSizeInByte" = EXCLUDED."fileSizeInByte",
                   orientation = EXCLUDED.orientation,
                   "dateTimeOriginal" = EXCLUDED."dateTimeOriginal",
                   "modifyDate" = EXCLUDED."modifyDate",
                   "timeZone" = EXCLUDED."timeZone",
                   latitude = EXCLUDED.latitude,
                   longitude = EXCLUDED.longitude,
                   city = EXCLUDED.city,
                   state = EXCLUDED.state,
                   country = EXCLUDED.country,
                   description = EXCLUDED.description,
                   "fNumber" = EXCLUDED."fNumber",
                   "focalLength" = EXCLUDED."focalLength",
                   iso = EXCLUDED.iso,
                   "exposureTime" = EXCLUDED."exposureTime",
                   "lensModel" = EXCLUDED."lensModel",
                   "projectionType" = EXCLUDED."projectionType",
                   rating = EXCLUDED.rating,
                   fps = EXCLUDED.fps"#,
        )
        .bind(exif.asset_id)
        .bind(&exif.make)
        .bind(&exif.model)
        .bind(exif.exif_image_width)
        .bind(exif.exif_image_height)
        .bind(exif.file_size_in_byte)
        .bind(&exif.orientation)
        .bind(exif.date_time_original)
        .bind(exif.modify_date)
        .bind(&exif.time_zone)
        .bind(exif.latitude)
        .bind(exif.longitude)
        .bind(&exif.city)
        .bind(&exif.state)
        .bind(&exif.country)
        .bind(&exif.description)
        .bind(exif.f_number)
        .bind(exif.focal_length)
        .bind(exif.iso)
        .bind(&exif.exposure_time)
        .bind(&exif.lens_model)
        .bind(&exif.projection_type)
        .bind(exif.rating)
        .bind(exif.fps)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    pub async fn update_duration(
        &self,
        asset_id: Uuid,
        duration_millis: Option<i32>,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE asset SET duration = $2, "updatedAt" = datetime('now') WHERE id = $1"#,
        )
        .bind(asset_id)
        .bind(duration_millis)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    pub async fn update_thumbhash(&self, asset_id: Uuid, thumbhash: &[u8]) -> Result<()> {
        sqlx::query(
            r#"UPDATE asset SET thumbhash = $2, "updatedAt" = datetime('now') WHERE id = $1"#,
        )
        .bind(asset_id)
        .bind(thumbhash)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    pub async fn asset_file_path(&self, asset_id: Uuid, file_type: &str) -> Result<Option<String>> {
        sqlx::query_scalar(r#"SELECT path FROM asset_file WHERE "assetId" = $1 AND type = $2"#)
            .bind(asset_id)
            .bind(file_type)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)
    }

    pub async fn upsert_asset_file(
        &self,
        asset_id: Uuid,
        file_type: &str,
        path: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO asset_file ("assetId", type, path)
               VALUES ($1, $2, $3)
               ON CONFLICT ("assetId", type) DO UPDATE
               SET path = EXCLUDED.path, "updatedAt" = datetime('now')"#,
        )
        .bind(asset_id)
        .bind(file_type)
        .bind(path)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    pub async fn mark_metadata_extracted(&self, asset_id: Uuid) -> Result<()> {
        self.mark_job_status(asset_id, r#""metadataExtractedAt""#)
            .await
    }

    pub async fn mark_preview_generated(&self, asset_id: Uuid) -> Result<()> {
        self.mark_job_status(asset_id, r#""previewAt""#).await
    }

    pub async fn mark_thumbnail_generated(&self, asset_id: Uuid) -> Result<()> {
        self.mark_job_status(asset_id, r#""thumbnailAt""#).await
    }

    async fn mark_job_status(&self, asset_id: Uuid, column: &str) -> Result<()> {
        let sql = format!(
            r#"INSERT INTO asset_job_status ("assetId", {column})
               VALUES ($1, datetime('now'))
               ON CONFLICT ("assetId") DO UPDATE SET {column} = datetime('now')"#
        );
        sqlx::query(&sql)
            .bind(asset_id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    pub async fn permanently_delete(&self, asset_id: Uuid) -> Result<()> {
        sqlx::query(r#"DELETE FROM asset WHERE id = $1"#)
            .bind(asset_id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}

const ASSET_SELECT_SQL: &str = r#"SELECT id, "ownerId", "libraryId", lower(hex(id)) AS "deviceAssetId", 'domus' AS "deviceId", type,
       "originalPath", "originalFileName", checksum, visibility, "isFavorite", "isOffline", "isExternal",
       "livePhotoVideoId", "stackId", CAST(duration AS TEXT) AS duration, thumbhash, "fileCreatedAt", "fileModifiedAt", "localDateTime",
       "createdAt", "updatedAt", "deletedAt"
FROM asset WHERE id = $1"#;

fn parent_folder(path: &str) -> Option<String> {
    path.rsplit_once('/')
        .map(|(folder, _)| folder.to_owned())
        .filter(|folder| !folder.is_empty())
}

fn asset_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Asset> {
    Ok(Asset {
        id: row.try_get("id").map_err(db_err)?,
        owner_id: row.try_get("ownerId").map_err(db_err)?,
        library_id: row.try_get("libraryId").map_err(db_err)?,
        device_asset_id: row.try_get("deviceAssetId").map_err(db_err)?,
        device_id: row.try_get("deviceId").map_err(db_err)?,
        asset_type: asset_type_from_db(row.try_get::<String, _>("type").map_err(db_err)?.as_str())?,
        original_path: row.try_get("originalPath").map_err(db_err)?,
        original_file_name: row.try_get("originalFileName").map_err(db_err)?,
        checksum: row.try_get("checksum").map_err(db_err)?,
        visibility: visibility_from_db(
            row.try_get::<String, _>("visibility")
                .map_err(db_err)?
                .as_str(),
        )?,
        is_favorite: row.try_get("isFavorite").map_err(db_err)?,
        is_offline: row.try_get("isOffline").map_err(db_err)?,
        is_external: row.try_get("isExternal").map_err(db_err)?,
        live_photo_video_id: row.try_get("livePhotoVideoId").map_err(db_err)?,
        stack_id: row.try_get("stackId").map_err(db_err)?,
        duration: normalize_duration(row.try_get("duration").map_err(db_err)?),
        thumbhash: row.try_get("thumbhash").map_err(db_err)?,
        file_created_at: row.try_get("fileCreatedAt").map_err(db_err)?,
        file_modified_at: row.try_get("fileModifiedAt").map_err(db_err)?,
        local_date_time: row.try_get("localDateTime").map_err(db_err)?,
        created_at: row.try_get("createdAt").map_err(db_err)?,
        updated_at: row.try_get("updatedAt").map_err(db_err)?,
        deleted_at: row.try_get("deletedAt").map_err(db_err)?,
    })
}

fn normalize_duration(value: Option<String>) -> Option<String> {
    let value = value?;
    if value.contains(':') {
        return Some(value);
    }
    let millis = value.parse::<i64>().ok()?;
    Some(duration_string_from_millis(millis))
}

fn duration_string_from_millis(millis: i64) -> String {
    let millis = millis.max(0);
    let total_seconds = millis / 1000;
    let ms = millis % 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{ms:03}")
}

fn asset_type_from_db(value: &str) -> Result<AssetType> {
    match value {
        "IMAGE" => Ok(AssetType::Image),
        "VIDEO" => Ok(AssetType::Video),
        "AUDIO" => Ok(AssetType::Audio),
        "OTHER" => Ok(AssetType::Other),
        other => Err(Error::Database(format!("unknown asset type: {other}"))),
    }
}

fn asset_type_to_db(value: AssetType) -> &'static str {
    match value {
        AssetType::Image => "IMAGE",
        AssetType::Video => "VIDEO",
        AssetType::Audio => "AUDIO",
        AssetType::Other => "OTHER",
    }
}

fn visibility_from_db(value: &str) -> Result<AssetVisibility> {
    match value {
        "timeline" => Ok(AssetVisibility::Timeline),
        "archive" => Ok(AssetVisibility::Archive),
        "hidden" => Ok(AssetVisibility::Hidden),
        "locked" => Ok(AssetVisibility::Locked),
        other => Err(Error::Database(format!(
            "unknown asset visibility: {other}"
        ))),
    }
}

fn visibility_to_db(value: AssetVisibility) -> &'static str {
    match value {
        AssetVisibility::Timeline => "timeline",
        AssetVisibility::Archive => "archive",
        AssetVisibility::Hidden => "hidden",
        AssetVisibility::Locked => "locked",
    }
}

#[derive(sqlx::FromRow)]
struct ExifRow {
    asset_id: Uuid,
    make: Option<String>,
    model: Option<String>,
    exif_image_width: Option<i32>,
    exif_image_height: Option<i32>,
    file_size_in_byte: Option<i64>,
    orientation: Option<String>,
    date_time_original: Option<chrono::DateTime<chrono::Utc>>,
    modify_date: Option<chrono::DateTime<chrono::Utc>>,
    time_zone: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    description: Option<String>,
    f_number: Option<f64>,
    focal_length: Option<f64>,
    iso: Option<i32>,
    exposure_time: Option<String>,
    lens_model: Option<String>,
    projection_type: Option<String>,
    rating: Option<i32>,
    fps: Option<f64>,
}

impl From<ExifRow> for Exif {
    fn from(row: ExifRow) -> Self {
        Self {
            asset_id: row.asset_id,
            make: row.make,
            model: row.model,
            exif_image_width: row.exif_image_width,
            exif_image_height: row.exif_image_height,
            file_size_in_byte: row.file_size_in_byte,
            orientation: row.orientation,
            date_time_original: row.date_time_original,
            modify_date: row.modify_date,
            time_zone: row.time_zone,
            latitude: row.latitude,
            longitude: row.longitude,
            city: row.city,
            state: row.state,
            country: row.country,
            description: row.description,
            f_number: row.f_number,
            focal_length: row.focal_length,
            iso: row.iso,
            exposure_time: row.exposure_time,
            lens_model: row.lens_model,
            projection_type: row.projection_type,
            rating: row.rating,
            fps: row.fps,
        }
    }
}
