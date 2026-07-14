//! Album CRUD, sharing (album_users) and album assets.

#[allow(unused_imports)]
use domus_common::{Error, Result};
use domus_db::Repositories;
#[allow(unused_imports)]
use uuid::Uuid;

pub struct AlbumService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl AlbumService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn list(&self, user_id: Uuid, shared: Option<bool>) -> Result<Vec<domus_db::entities::Album>> {
        self.repos.album.list_for_user(user_id, shared).await
    }

    pub async fn get(&self, id: Uuid) -> Result<domus_db::entities::Album> {
        self.repos.album.get(id).await
    }

    pub async fn create(&self, owner_id: Uuid, name: &str, description: &str) -> Result<domus_db::entities::Album> {
        self.repos.album.create(owner_id, name, description).await
    }

    pub async fn add_assets(&self, album_id: Uuid, asset_ids: &[Uuid]) -> Result<()> {
        self.repos.album.add_assets(album_id, asset_ids).await
    }

    pub async fn remove_assets(&self, album_id: Uuid, asset_ids: &[Uuid]) -> Result<()> {
        self.repos.album.remove_assets(album_id, asset_ids).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        self.repos.album.delete(id).await
    }

    pub async fn statistics(&self, _user_id: Uuid) -> Result<serde_json::Value> {
        Err(Error::NotImplemented("AlbumService::statistics"))
    }
}
