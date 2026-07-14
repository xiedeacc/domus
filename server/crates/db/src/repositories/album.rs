use crate::entities::Album;
use domus_common::{Error, Result};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AlbumRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl AlbumRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, _id: Uuid) -> Result<Album> {
        Err(Error::NotImplemented("AlbumRepository::get"))
    }

    /// Albums owned by the user plus albums shared with them.
    pub async fn list_for_user(&self, _user_id: Uuid, _shared: Option<bool>) -> Result<Vec<Album>> {
        Err(Error::NotImplemented("AlbumRepository::list_for_user"))
    }

    pub async fn create(&self, _owner_id: Uuid, _name: &str, _description: &str) -> Result<Album> {
        Err(Error::NotImplemented("AlbumRepository::create"))
    }

    pub async fn add_assets(&self, _album_id: Uuid, _asset_ids: &[Uuid]) -> Result<()> {
        Err(Error::NotImplemented("AlbumRepository::add_assets"))
    }

    pub async fn remove_assets(&self, _album_id: Uuid, _asset_ids: &[Uuid]) -> Result<()> {
        Err(Error::NotImplemented("AlbumRepository::remove_assets"))
    }

    pub async fn delete(&self, _id: Uuid) -> Result<()> {
        Err(Error::NotImplemented("AlbumRepository::delete"))
    }
}
