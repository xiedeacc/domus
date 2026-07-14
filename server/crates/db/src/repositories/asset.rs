use super::db_err;
use crate::entities::{Asset, Exif};
use domus_common::{Error, Result};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AssetRepository {
    pool: PgPool,
}

pub struct CreateAsset {
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

    pub async fn get(&self, _id: Uuid) -> Result<Asset> {
        Err(Error::NotImplemented("AssetRepository::get"))
    }

    pub async fn get_exif(&self, _asset_id: Uuid) -> Result<Option<Exif>> {
        Err(Error::NotImplemented("AssetRepository::get_exif"))
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

    pub async fn create(&self, _asset: CreateAsset) -> Result<Asset> {
        Err(Error::NotImplemented("AssetRepository::create"))
    }

    pub async fn update_favorite(&self, _ids: &[Uuid], _is_favorite: bool) -> Result<()> {
        Err(Error::NotImplemented("AssetRepository::update_favorite"))
    }

    /// Soft-delete: move assets to trash (sets deletedAt).
    pub async fn trash(&self, _ids: &[Uuid]) -> Result<()> {
        Err(Error::NotImplemented("AssetRepository::trash"))
    }

    pub async fn restore(&self, _ids: &[Uuid]) -> Result<()> {
        Err(Error::NotImplemented("AssetRepository::restore"))
    }

    pub async fn statistics(&self, _user_id: Uuid) -> Result<(i64, i64)> {
        Err(Error::NotImplemented("AssetRepository::statistics"))
    }
}
