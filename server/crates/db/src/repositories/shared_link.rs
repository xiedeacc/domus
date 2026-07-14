use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;
use crate::entities::SharedLink;

#[derive(Clone)]
pub struct SharedLinkRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl SharedLinkRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_key(&self, _key: &[u8]) -> Result<Option<SharedLink>> {
        Err(Error::NotImplemented("SharedLinkRepository::get_by_key"))
    }

    pub async fn get_by_slug(&self, _slug: &str) -> Result<Option<SharedLink>> {
        Err(Error::NotImplemented("SharedLinkRepository::get_by_slug"))
    }

    pub async fn list_for_user(&self, _user_id: Uuid) -> Result<Vec<SharedLink>> {
        Err(Error::NotImplemented("SharedLinkRepository::list_for_user"))
    }

    pub async fn create(&self, _link: SharedLink) -> Result<SharedLink> {
        Err(Error::NotImplemented("SharedLinkRepository::create"))
    }

    pub async fn delete(&self, _id: Uuid) -> Result<()> {
        Err(Error::NotImplemented("SharedLinkRepository::delete"))
    }
}
