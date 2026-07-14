use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;
use crate::entities::Tag;

#[derive(Clone)]
pub struct TagRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl TagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_user(&self, _user_id: Uuid) -> Result<Vec<Tag>> {
        Err(Error::NotImplemented("TagRepository::list_for_user"))
    }

    pub async fn upsert_value(&self, _user_id: Uuid, _value: &str) -> Result<Tag> {
        Err(Error::NotImplemented("TagRepository::upsert_value"))
    }

    pub async fn tag_assets(&self, _tag_id: Uuid, _asset_ids: &[Uuid]) -> Result<()> {
        Err(Error::NotImplemented("TagRepository::tag_assets"))
    }

    pub async fn delete(&self, _id: Uuid) -> Result<()> {
        Err(Error::NotImplemented("TagRepository::delete"))
    }
}
