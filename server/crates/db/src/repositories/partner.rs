use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;
use crate::entities::Partner;

#[derive(Clone)]
pub struct PartnerRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl PartnerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, _user_id: Uuid) -> Result<Vec<Partner>> {
        Err(Error::NotImplemented("PartnerRepository::list"))
    }

    pub async fn create(&self, _shared_by: Uuid, _shared_with: Uuid) -> Result<Partner> {
        Err(Error::NotImplemented("PartnerRepository::create"))
    }

    pub async fn remove(&self, _shared_by: Uuid, _shared_with: Uuid) -> Result<()> {
        Err(Error::NotImplemented("PartnerRepository::remove"))
    }
}
