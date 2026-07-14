//! Trash: restore or permanently empty soft-deleted assets.

use domus_common::Result;
use domus_db::Repositories;
use domus_jobs::{JobName, PgJobQueue};
use uuid::Uuid;

pub struct TrashService {
    repos: Repositories,
    queue: PgJobQueue,
}

impl TrashService {
    pub fn new(repos: Repositories, queue: PgJobQueue) -> Self {
        Self { repos, queue }
    }

    pub async fn restore_all(&self, user_id: Uuid) -> Result<u64> {
        self.repos.asset.restore_all_for_user(user_id).await
    }

    pub async fn restore(&self, ids: &[Uuid]) -> Result<()> {
        self.repos.asset.restore(ids).await
    }

    /// Enqueue permanent deletion of everything in the trash.
    pub async fn empty(&self, user_id: Uuid) -> Result<u64> {
        let ids = self.repos.asset.trashed_ids_for_user(user_id).await?;
        for id in &ids {
            self.queue
                .enqueue(JobName::AssetDeletion, serde_json::json!({ "id": id }))
                .await?;
        }
        Ok(ids.len() as u64)
    }
}
