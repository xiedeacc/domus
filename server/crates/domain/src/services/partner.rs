//! Partner sharing: expose one user's timeline to another.

#[allow(unused_imports)]
use domus_common::{Error, Result};
use domus_db::Repositories;
#[allow(unused_imports)]
use uuid::Uuid;

pub struct PartnerService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl PartnerService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<domus_db::entities::Partner>> {
        self.repos.partner.list(user_id).await
    }

    pub async fn create(
        &self,
        shared_by: Uuid,
        shared_with: Uuid,
    ) -> Result<domus_db::entities::Partner> {
        self.repos.partner.create(shared_by, shared_with).await
    }

    pub async fn remove(&self, shared_by: Uuid, shared_with: Uuid) -> Result<()> {
        self.repos.partner.remove(shared_by, shared_with).await
    }

    pub async fn update_timeline(
        &self,
        shared_by: Uuid,
        shared_with: Uuid,
        in_timeline: bool,
    ) -> Result<domus_db::entities::Partner> {
        self.repos
            .partner
            .update_timeline(shared_by, shared_with, in_timeline)
            .await
    }
}
