//! Metadata search (SQL filters). Smart/CLIP search is intentionally unsupported (no ML service) and reports as disabled in server features.

#[allow(unused_imports)]
use domus_common::{Error, Result};
use domus_db::Repositories;
#[allow(unused_imports)]
use uuid::Uuid;

pub struct SearchService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl SearchService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn search_metadata(&self, user_id: Uuid, filters: serde_json::Value) -> Result<serde_json::Value> {
        self.repos.search.search_metadata(&[user_id], filters).await
    }

    pub async fn suggestions(&self, user_id: Uuid, kind: &str) -> Result<Vec<String>> {
        self.repos.search.suggestions(user_id, kind).await
    }

    pub async fn explore(&self, user_id: Uuid) -> Result<serde_json::Value> {
        self.repos.search.explore(user_id).await
    }
}
