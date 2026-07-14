//! Comments and likes on shared album assets.

#[allow(unused_imports)]
use domus_common::{Error, Result};
use domus_db::Repositories;
#[allow(unused_imports)]
use uuid::Uuid;

pub struct ActivityService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl ActivityService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn list(
        &self,
        album_id: Uuid,
        asset_id: Option<Uuid>,
    ) -> Result<Vec<domus_db::entities::Activity>> {
        self.repos.activity.list_for_album(album_id, asset_id).await
    }
}
