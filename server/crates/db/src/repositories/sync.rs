use crate::PgPool;
use domus_common::{Error, Result};
#[allow(unused_imports)]
use uuid::Uuid;

#[derive(Clone)]
pub struct SyncRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl SyncRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_checkpoints(&self, _session_id: Uuid) -> Result<Vec<(String, String)>> {
        Err(Error::NotImplemented("SyncRepository::get_checkpoints"))
    }

    pub async fn upsert_checkpoints(&self, _session_id: Uuid, _acks: &[String]) -> Result<()> {
        Err(Error::NotImplemented("SyncRepository::upsert_checkpoints"))
    }

    pub async fn delete_checkpoints(&self, _session_id: Uuid, _types: &[String]) -> Result<()> {
        Err(Error::NotImplemented("SyncRepository::delete_checkpoints"))
    }
}
