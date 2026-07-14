//! Asset metadata operations (favorite, archive, description, bulk update,
//! delete). Binary up/download lives in `asset_media`.

use domus_common::types::AssetVisibility;
use domus_common::{Error, Result};
use domus_db::entities::{Asset, Exif};
use domus_db::Repositories;
use domus_jobs::{JobName, PgJobQueue};
use uuid::Uuid;

pub struct AssetService {
    repos: Repositories,
    queue: PgJobQueue,
}

impl AssetService {
    pub fn new(repos: Repositories, queue: PgJobQueue) -> Self {
        Self { repos, queue }
    }

    pub async fn get(&self, id: Uuid) -> Result<(Asset, Option<Exif>)> {
        let asset = self.repos.asset.get(id).await?;
        let exif = self.repos.asset.get_exif(id).await?;
        Ok((asset, exif))
    }

    pub async fn update(&self, id: Uuid, update: serde_json::Value) -> Result<Asset> {
        self.apply_update(&[id], update).await?;
        self.repos.asset.get(id).await
    }

    pub async fn bulk_update(&self, ids: &[Uuid], update: serde_json::Value) -> Result<()> {
        self.apply_update(ids, update).await
    }

    /// Soft-delete into trash; `force` skips trash and enqueues deletion jobs.
    pub async fn delete(&self, ids: &[Uuid], force: bool) -> Result<()> {
        if force {
            for id in ids {
                self.queue
                    .enqueue(JobName::AssetDeletion, serde_json::json!({ "id": id }))
                    .await?;
            }
            Ok(())
        } else {
            self.repos.asset.trash(ids).await
        }
    }

    pub async fn statistics(&self, user_id: Uuid) -> Result<(i64, i64)> {
        self.repos.asset.statistics(user_id).await
    }

    pub async fn device_asset_ids(&self, user_id: Uuid, device_id: &str) -> Result<Vec<String>> {
        self.repos
            .asset
            .list_device_asset_ids_by_device(user_id, device_id)
            .await
    }

    pub async fn unique_original_folders(&self, user_id: Uuid) -> Result<Vec<String>> {
        self.repos.asset.unique_original_folders(user_id).await
    }

    pub async fn assets_by_original_folder(
        &self,
        user_id: Uuid,
        folder: &str,
    ) -> Result<Vec<Asset>> {
        self.repos
            .asset
            .list_by_original_folder(user_id, folder)
            .await
    }

    pub async fn map_markers(&self, user_id: Uuid) -> Result<Vec<(Asset, Exif)>> {
        self.repos.asset.map_markers(user_id).await
    }

    /// Re-run parts of the pipeline for given assets (regenerate thumbnails,
    /// refresh metadata, transcode) — backs POST /assets/jobs.
    pub async fn run_job(&self, ids: &[Uuid], job_name: &str) -> Result<()> {
        let job = match job_name {
            "metadataExtraction" | "MetadataExtraction" => JobName::MetadataExtraction,
            "generatePreview" | "GeneratePreview" => JobName::GeneratePreview,
            "generateThumbnail" | "GenerateThumbnail" => JobName::GenerateThumbnail,
            "videoConversion" | "VideoConversion" => JobName::VideoConversion,
            _ => return Err(Error::BadRequest(format!("unknown asset job: {job_name}"))),
        };
        for id in ids {
            self.queue
                .enqueue(job.clone(), serde_json::json!({ "id": id }))
                .await?;
        }
        Ok(())
    }

    async fn apply_update(&self, ids: &[Uuid], update: serde_json::Value) -> Result<()> {
        if let Some(is_favorite) = update.get("isFavorite").and_then(|v| v.as_bool()) {
            self.repos.asset.update_favorite(ids, is_favorite).await?;
        }
        if let Some(is_archived) = update.get("isArchived").and_then(|v| v.as_bool()) {
            let visibility = if is_archived {
                AssetVisibility::Archive
            } else {
                AssetVisibility::Timeline
            };
            self.repos.asset.update_visibility(ids, visibility).await?;
        }
        if let Some(visibility) = update.get("visibility").and_then(|v| v.as_str()) {
            self.repos
                .asset
                .update_visibility(ids, parse_visibility(visibility)?)
                .await?;
        }
        Ok(())
    }
}

fn parse_visibility(value: &str) -> Result<AssetVisibility> {
    match value {
        "timeline" => Ok(AssetVisibility::Timeline),
        "archive" => Ok(AssetVisibility::Archive),
        "hidden" => Ok(AssetVisibility::Hidden),
        "locked" => Ok(AssetVisibility::Locked),
        _ => Err(Error::BadRequest(format!("invalid visibility: {value}"))),
    }
}
