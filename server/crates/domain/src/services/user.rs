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

    pub async fn create_admin_user(
        &self,
        email: &str,
        password: &str,
        name: &str,
        is_admin: bool,
    ) -> Result<domus_db::entities::User> {
        validate_email(email)?;
        let hash = bcrypt::hash(password, 10).map_err(|e| Error::Internal(e.into()))?;
        self.repos.user.create(email, &hash, name, is_admin).await
    }

    pub async fn update_profile(
        &self,
        _id: Uuid,
        _update: serde_json::Value,
    ) -> Result<domus_db::entities::User> {
        Err(Error::NotImplemented("UserService::update_profile"))
    }

    pub async fn set_license(&self, _id: Uuid, _license: serde_json::Value) -> Result<()> {
        Err(Error::NotImplemented("UserService::set_license"))
    }
}

pub(crate) fn validate_email(email: &str) -> Result<()> {
    let Some((local, domain)) = email.split_once('@') else {
        return Err(Error::BadRequest(
            "email must be a valid email address".into(),
        ));
    };
    if local.is_empty()
        || domain.is_empty()
        || email.chars().any(char::is_whitespace)
        || email.matches('@').count() != 1
    {
        return Err(Error::BadRequest(
            "email must be a valid email address".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_email_matches_immich_user_dto_expectations() {
        assert!(validate_email("valid@email.com").is_ok());
        assert!(validate_email("test@test").is_ok());
        assert!(validate_email("invalid email").is_err());
        assert!(validate_email("").is_err());
        assert!(validate_email("missing-at.example.com").is_err());
        assert!(validate_email("too@many@example.com").is_err());
    }
}
