//! User profile and admin user management.

#[allow(unused_imports)]
use domus_common::{Error, Result};
use domus_db::Repositories;
#[allow(unused_imports)]
use uuid::Uuid;

pub struct UserService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl UserService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn get(&self, id: Uuid) -> Result<domus_db::entities::User> {
        self.repos.user.get(id).await
    }

    pub async fn list(&self) -> Result<Vec<domus_db::entities::User>> {
        self.repos.user.list().await
    }

    pub async fn update_profile(&self, _id: Uuid, _update: serde_json::Value) -> Result<domus_db::entities::User> {
        Err(Error::NotImplemented("UserService::update_profile"))
    }

    pub async fn set_license(&self, _id: Uuid, _license: serde_json::Value) -> Result<()> {
        Err(Error::NotImplemented("UserService::set_license"))
    }
}
