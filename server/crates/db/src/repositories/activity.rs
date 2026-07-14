use crate::entities::Activity;
use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;

#[derive(Clone)]
pub struct ActivityRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl ActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_album(
        &self,
        _album_id: Uuid,
        _asset_id: Option<Uuid>,
    ) -> Result<Vec<Activity>> {
        Err(Error::NotImplemented("ActivityRepository::list_for_album"))
    }

    pub async fn create(&self, _activity: Activity) -> Result<Activity> {
        Err(Error::NotImplemented("ActivityRepository::create"))
    }

    pub async fn delete(&self, _id: Uuid) -> Result<()> {
        Err(Error::NotImplemented("ActivityRepository::delete"))
    }
}
