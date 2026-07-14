//! Asset metadata operations (favorite, archive, description, bulk update,
//! delete). Binary up/download lives in `asset_media`.

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

    pub async fn update(&self, _id: Uuid, _update: serde_json::Value) -> Result<Asset> {
        Err(Error::NotImplemented("AssetService::update"))
    }

    pub async fn bulk_update(&self, _ids: &[Uuid], _update: serde_json::Value) -> Result<()> {
        Err(Error::NotImplemented("AssetService::bulk_update"))
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

    /// Re-run parts of the pipeline for given assets (regenerate thumbnails,
    /// refresh metadata, transcode) — backs POST /assets/jobs.
    pub async fn run_job(&self, _ids: &[Uuid], _job_name: &str) -> Result<()> {
        Err(Error::NotImplemented("AssetService::run_job"))
    }
}
