use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;
use crate::entities::Memory;

#[derive(Clone)]
pub struct MemoryRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl MemoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_user(&self, _user_id: Uuid) -> Result<Vec<Memory>> {
        Err(Error::NotImplemented("MemoryRepository::list_for_user"))
    }

    pub async fn get(&self, _id: Uuid) -> Result<Memory> {
        Err(Error::NotImplemented("MemoryRepository::get"))
    }

    pub async fn create_on_this_day(&self, _user_id: Uuid) -> Result<Vec<Memory>> {
        Err(Error::NotImplemented("MemoryRepository::create_on_this_day"))
    }
}
