use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;

#[derive(Clone)]
pub struct SearchRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl SearchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search_metadata(&self, _user_ids: &[Uuid], _filters: serde_json::Value) -> Result<serde_json::Value> {
        Err(Error::NotImplemented("SearchRepository::search_metadata"))
    }

    pub async fn suggestions(&self, _user_id: Uuid, _kind: &str) -> Result<Vec<String>> {
        Err(Error::NotImplemented("SearchRepository::suggestions"))
    }

    pub async fn explore(&self, _user_id: Uuid) -> Result<serde_json::Value> {
        Err(Error::NotImplemented("SearchRepository::explore"))
    }
}
