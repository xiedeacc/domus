use crate::PgPool;
use domus_common::{Error, Result};
#[allow(unused_imports)]
use uuid::Uuid;

#[derive(Clone)]
pub struct JobRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl JobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue(
        &self,
        _queue: &str,
        _name: &str,
        _data: serde_json::Value,
    ) -> Result<Uuid> {
        Err(Error::NotImplemented("JobRepository::enqueue"))
    }

    pub async fn dequeue(&self, _queue: &str) -> Result<Option<(Uuid, String, serde_json::Value)>> {
        Err(Error::NotImplemented("JobRepository::dequeue"))
    }

    pub async fn complete(&self, _id: Uuid) -> Result<()> {
        Err(Error::NotImplemented("JobRepository::complete"))
    }

    pub async fn fail(&self, _id: Uuid, _error: &str) -> Result<()> {
        Err(Error::NotImplemented("JobRepository::fail"))
    }

    pub async fn counts(&self, _queue: &str) -> Result<serde_json::Value> {
        Err(Error::NotImplemented("JobRepository::counts"))
    }
}
