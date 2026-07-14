use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;

#[derive(Clone)]
pub struct SystemMetadataRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl SystemMetadataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, _key: &str) -> Result<Option<serde_json::Value>> {
        Err(Error::NotImplemented("SystemMetadataRepository::get"))
    }

    pub async fn set(&self, _key: &str, _value: serde_json::Value) -> Result<()> {
        Err(Error::NotImplemented("SystemMetadataRepository::set"))
    }
}
