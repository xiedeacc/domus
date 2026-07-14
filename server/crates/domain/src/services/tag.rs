//! Hierarchical tags (value = "a/b/c" path).

use domus_common::Result;
use domus_db::Repositories;
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

    pub async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        parent_id: Option<Uuid>,
        color: Option<&str>,
    ) -> Result<domus_db::entities::Tag> {
        let value = if let Some(parent_id) = parent_id {
            let parent = self.repos.tag.get(parent_id).await?;
            format!("{}/{}", parent.value, name)
        } else {
            name.to_owned()
        };
        self.repos
            .tag
            .create(user_id, &value, parent_id, color)
            .await
    }

    pub async fn get(&self, id: Uuid) -> Result<domus_db::entities::Tag> {
        self.repos.tag.get(id).await
    }

    pub async fn update_color(
        &self,
        id: Uuid,
        color: Option<&str>,
    ) -> Result<domus_db::entities::Tag> {
        self.repos.tag.update_color(id, color).await
    }

    pub async fn tag_assets(&self, tag_id: Uuid, asset_ids: &[Uuid]) -> Result<u64> {
        self.repos.tag.tag_assets(tag_id, asset_ids).await
    }

    pub async fn untag_assets(&self, tag_id: Uuid, asset_ids: &[Uuid]) -> Result<u64> {
        self.repos.tag.untag_assets(tag_id, asset_ids).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        self.repos.tag.delete(id).await
    }
}
