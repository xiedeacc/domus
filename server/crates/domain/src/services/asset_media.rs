//! Asset binary ingest and serving: upload, original/thumbnail download,
//! video playback and replacement.

use domus_common::{Error, Result};
use domus_db::Repositories;
use domus_jobs::{JobName, PgJobQueue};
use domus_media::storage::StorageCore;
use std::path::PathBuf;
use uuid::Uuid;

pub struct AssetMediaService {
    repos: Repositories,
    queue: PgJobQueue,
    #[allow(dead_code)]
    storage: StorageCore,
}

pub struct UploadRequest {
    pub owner_id: Uuid,
    pub device_asset_id: String,
    pub device_id: String,
    pub file_created_at: chrono::DateTime<chrono::Utc>,
    pub file_modified_at: chrono::DateTime<chrono::Utc>,
    pub filename: String,
    pub is_favorite: bool,
    /// Temp file the multipart body was streamed to.
    pub staged_file: PathBuf,
    pub checksum: Vec<u8>,
}

pub enum UploadOutcome {
    /// 201 — new asset created.
    Created(Uuid),
    /// 200 — checksum matched an existing asset (client should not re-upload).
    Duplicate(Uuid),
}

impl AssetMediaService {
    pub fn new(repos: Repositories, queue: PgJobQueue, storage: StorageCore) -> Self {
        Self { repos, queue, storage }
    }

    /// Ingest an uploaded file:
    ///   1. dedup by (owner, SHA-1) — return Duplicate without touching disk
    ///   2. move the staged file into upload/<user>/<xx>/<yy>/<uuid>.<ext>
    ///   3. insert the asset row
    ///   4. enqueue MetadataExtraction (which fans out the rest of the
    ///      pipeline: thumbnails → thumbhash → video conversion)
    pub async fn upload(&self, request: UploadRequest) -> Result<UploadOutcome> {
        if let Some(existing) = self
            .repos
            .asset
            .get_by_checksum(request.owner_id, &request.checksum)
            .await?
        {
            return Ok(UploadOutcome::Duplicate(existing));
        }
        // TODO: steps 2-3 (move file, insert row) once AssetRepository::create lands.
        let _ = &self.queue;
        let _ = JobName::MetadataExtraction;
        Err(Error::NotImplemented("AssetMediaService::upload"))
    }

    /// Path of the original file for GET /assets/:id/original.
    pub async fn original_path(&self, _asset_id: Uuid) -> Result<PathBuf> {
        Err(Error::NotImplemented("AssetMediaService::original_path"))
    }

    /// Path of preview/thumbnail for GET /assets/:id/thumbnail?size=...
    pub async fn thumbnail_path(&self, _asset_id: Uuid, _size: &str) -> Result<PathBuf> {
        Err(Error::NotImplemented("AssetMediaService::thumbnail_path"))
    }

    /// Path of the playback file (encoded video if present, else original)
    /// for GET /assets/:id/video/playback.
    pub async fn playback_path(&self, _asset_id: Uuid) -> Result<PathBuf> {
        Err(Error::NotImplemented("AssetMediaService::playback_path"))
    }

    /// POST /assets/exist + POST /assets/bulk-upload-check support.
    pub async fn check_existing(&self, _owner_id: Uuid, _device_asset_ids: &[String]) -> Result<Vec<String>> {
        Err(Error::NotImplemented("AssetMediaService::check_existing"))
    }
}
