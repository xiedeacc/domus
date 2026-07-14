//! Hierarchical tags (value = "a/b/c" path).

#[allow(unused_imports)]
use domus_common::{Error, Result};
use domus_db::Repositories;
#[allow(unused_imports)]
use uuid::Uuid;

pub struct TagService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl TagService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<domus_db::entities::Tag>> {
        self.repos.tag.list_for_user(user_id).await
    }

    pub async fn upsert(&self, user_id: Uuid, value: &str) -> Result<domus_db::entities::Tag> {
        self.repos.tag.upsert_value(user_id, value).await
    }

    pub async fn tag_assets(&self, tag_id: Uuid, asset_ids: &[Uuid]) -> Result<()> {
        self.repos.tag.tag_assets(tag_id, asset_ids).await
    }
}
