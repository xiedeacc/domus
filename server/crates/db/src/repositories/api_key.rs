use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;
use crate::entities::ApiKey;

#[derive(Clone)]
pub struct ApiKeyRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_key_hash(&self, _key_hash: &str) -> Result<Option<ApiKey>> {
        Err(Error::NotImplemented("ApiKeyRepository::get_by_key_hash"))
    }

    pub async fn list_for_user(&self, _user_id: Uuid) -> Result<Vec<ApiKey>> {
        Err(Error::NotImplemented("ApiKeyRepository::list_for_user"))
    }

    pub async fn create(&self, _user_id: Uuid, _name: &str, _key_hash: &str, _permissions: &[String]) -> Result<ApiKey> {
        Err(Error::NotImplemented("ApiKeyRepository::create"))
    }

    pub async fn delete(&self, _id: Uuid) -> Result<()> {
        Err(Error::NotImplemented("ApiKeyRepository::delete"))
    }
}
