//! Asset stacks (burst shots grouped under a primary asset).

#[allow(unused_imports)]
use domus_common::{Error, Result};
use domus_db::Repositories;
#[allow(unused_imports)]
use uuid::Uuid;

pub struct StackService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl StackService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<domus_db::entities::Stack>> {
        self.repos.stack.list_for_user(user_id).await
    }

    pub async fn create(
        &self,
        owner_id: Uuid,
        asset_ids: &[Uuid],
    ) -> Result<domus_db::entities::Stack> {
        self.repos.stack.create(owner_id, asset_ids).await
    }

    pub async fn get(&self, id: Uuid) -> Result<domus_db::entities::Stack> {
        self.repos.stack.get(id).await
    }

    pub async fn assets(&self, id: Uuid) -> Result<Vec<domus_db::entities::Asset>> {
        self.repos.asset.list_by_stack(id).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        self.repos.stack.delete(id).await
    }

    pub async fn remove_asset(&self, id: Uuid, asset_id: Uuid) -> Result<()> {
        self.repos.stack.remove_asset(id, asset_id).await
    }
}
