//! External libraries: watch folders on disk and import them without
//! copying (assets marked isExternal).

use domus_common::{Error, Result};
use domus_db::entities::Library;
use domus_db::Repositories;
use domus_jobs::{JobName, PgJobQueue};
use uuid::Uuid;

pub struct LibraryService {
    repos: Repositories,
    queue: PgJobQueue,
}

impl LibraryService {
    pub fn new(repos: Repositories, queue: PgJobQueue) -> Self {
        Self { repos, queue }
    }

    pub async fn list(&self, owner_id: Option<Uuid>) -> Result<Vec<Library>> {
        self.repos.library.list(owner_id).await
    }

    pub async fn create(&self, owner_id: Uuid, name: &str, import_paths: &[String]) -> Result<Library> {
        self.repos.library.create(owner_id, name, import_paths).await
    }

    /// Kick off a scan job over the library's import paths.
    pub async fn scan(&self, id: Uuid) -> Result<()> {
        self.queue
            .enqueue(JobName::LibraryScan, serde_json::json!({ "id": id }))
            .await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        self.repos.library.delete(id).await
    }

    pub async fn validate(&self, _id: Uuid, _import_paths: &[String]) -> Result<serde_json::Value> {
        Err(Error::NotImplemented("LibraryService::validate"))
    }
}
