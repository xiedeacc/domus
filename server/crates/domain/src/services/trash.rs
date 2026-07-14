//! Trash: restore or permanently empty soft-deleted assets.

use domus_common::{Error, Result};
use domus_db::Repositories;
use domus_jobs::PgJobQueue;
use uuid::Uuid;

pub struct TrashService {
    repos: Repositories,
    #[allow(dead_code)]
    queue: PgJobQueue,
}

impl TrashService {
    pub fn new(repos: Repositories, queue: PgJobQueue) -> Self {
        Self { repos, queue }
    }

    pub async fn restore_all(&self, _user_id: Uuid) -> Result<u64> {
        Err(Error::NotImplemented("TrashService::restore_all"))
    }

    pub async fn restore(&self, ids: &[Uuid]) -> Result<()> {
        self.repos.asset.restore(ids).await
    }

    /// Enqueue permanent deletion of everything in the trash.
    pub async fn empty(&self, _user_id: Uuid) -> Result<u64> {
        Err(Error::NotImplemented("TrashService::empty"))
    }
}
