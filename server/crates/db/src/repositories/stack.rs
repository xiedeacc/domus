use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;
use crate::entities::Stack;

#[derive(Clone)]
pub struct StackRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl StackRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_user(&self, _user_id: Uuid) -> Result<Vec<Stack>> {
        Err(Error::NotImplemented("StackRepository::list_for_user"))
    }

    pub async fn create(&self, _owner_id: Uuid, _asset_ids: &[Uuid]) -> Result<Stack> {
        Err(Error::NotImplemented("StackRepository::create"))
    }

    pub async fn delete(&self, _id: Uuid) -> Result<()> {
        Err(Error::NotImplemented("StackRepository::delete"))
    }
}
