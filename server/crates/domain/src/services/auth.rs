//! Authentication: password login, session issue/validate, API keys.
//!
//! Wire-compatibility notes (must match Immich exactly):
//!   - bearer token accepted via `Authorization: Bearer <t>`, cookie
//!     `immich_access_token`, or `x-immich-session-token` header
//!   - API keys via `x-api-key` header
//!   - session tokens are random bytes, base64url-encoded; the DB stores the
//!     SHA-256 hex hash, never the raw token
//!   - passwords hashed with bcrypt (existing Immich hashes keep working)

use domus_common::{Error, Result};
use domus_db::entities::{ApiKey, Session, User};
use domus_db::Repositories;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::shared_link::decode_shared_link_key;
use super::user::validate_email;

pub struct AuthService {
    repos: Repositories,
}

/// Resolved identity attached to each authenticated request.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub is_admin: bool,
    pub session_id: Option<Uuid>,
    pub api_key_id: Option<Uuid>,
    pub shared_link_id: Option<Uuid>,
}

pub struct LoginOutcome {
    pub user: User,
    pub token: String,
    pub session: Session,
}

pub struct ApiKeyCreateOutcome {
    pub api_key: ApiKey,
    pub secret: String,
}

pub struct OAuthProfile {
    pub oauth_id: String,
    pub email: String,
    pub name: String,
}

impl AuthService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
        device: (&str, &str),
    ) -> Result<LoginOutcome> {
        let user = self
            .repos
            .user
            .get_by_email(email)
            .await?
            .ok_or_else(|| Error::Unauthorized("Incorrect email or password".into()))?;
        let ok = bcrypt::verify(password, &user.password)
            .map_err(|_| Error::Unauthorized("Incorrect email or password".into()))?;
        if !ok {
            return Err(Error::Unauthorized("Incorrect email or password".into()));
        }
        self.issue_session(user, device).await
    }

    pub async fn oauth_login(&self, profile: OAuthProfile) -> Result<LoginOutcome> {
        let user = if let Some(user) = self.repos.user.get_by_oauth_id(&profile.oauth_id).await? {
            user
        } else if let Some(user) = self.repos.user.get_by_email(&profile.email).await? {
            if user.oauth_id.is_empty() {
                self.repos
                    .user
                    .set_oauth_id(user.id, &profile.oauth_id)
                    .await?
            } else {
                user
            }
        } else {
            let first_user_is_admin = self.repos.user.list().await?.is_empty();
            self.repos
                .user
                .create_oauth(
                    &profile.email,
                    &profile.name,
                    &profile.oauth_id,
                    first_user_is_admin,
                )
                .await?
        };
        self.issue_session(user, ("OAUTH", "")).await
    }

    pub async fn link_oauth_account(&self, user_id: Uuid, oauth_id: &str) -> Result<User> {
        self.repos.user.set_oauth_id(user_id, oauth_id).await
    }

    /// First user to register becomes the admin (Immich's onboarding rule).
    pub async fn admin_sign_up(&self, email: &str, password: &str, name: &str) -> Result<User> {
        validate_email(email)?;
        if self.repos.user.count_admins().await? > 0 {
            return Err(Error::BadRequest("The server already has an admin".into()));
        }
        let hash = bcrypt::hash(password, 10).map_err(|e| Error::Internal(e.into()))?;
        self.repos.user.create(email, &hash, name, true).await
    }

    async fn issue_session(&self, user: User, device: (&str, &str)) -> Result<LoginOutcome> {
        let token = Self::generate_token();
        let session = self
            .repos
            .session
            .create(user.id, &Self::hash_token(&token), device.0, device.1)
            .await?;
        Ok(LoginOutcome {
            user,
            token,
            session,
        })
    }

    pub async fn validate_session(&self, raw_token: &str) -> Result<AuthContext> {
        let session = self
            .repos
            .session
            .get_by_token_hash(&Self::hash_token(raw_token))
            .await?
            .ok_or_else(|| Error::Unauthorized("Invalid user token".into()))?;
        if let Some(expires) = session.expires_at {
            if expires < chrono::Utc::now() {
                return Err(Error::Unauthorized("Invalid user token".into()));
            }
        }
        let user = self.repos.user.get(session.user_id).await?;
        Ok(AuthContext {
            user_id: user.id,
            is_admin: user.is_admin,
            session_id: Some(session.id),
            api_key_id: None,
            shared_link_id: None,
        })
    }

    pub async fn validate_api_key(&self, raw_key: &str) -> Result<AuthContext> {
        let key = self
            .repos
            .api_key
            .get_by_key_hash(&Self::hash_token(raw_key))
            .await?
            .ok_or_else(|| Error::Unauthorized("Invalid API key".into()))?;
        let user = self.repos.user.get(key.user_id).await?;
        Ok(AuthContext {
            user_id: user.id,
            is_admin: user.is_admin,
            session_id: None,
            api_key_id: Some(key.id),
            shared_link_id: None,
        })
    }

    pub async fn validate_shared_link(
        &self,
        key: Option<&str>,
        slug: Option<&str>,
    ) -> Result<AuthContext> {
        let link = match (key, slug) {
            (Some(k), _) => {
                let bytes = decode_shared_link_key(k)?;
                self.repos
                    .shared_link
                    .get_by_key(&bytes)
                    .await?
                    .ok_or_else(|| Error::Unauthorized("invalid share key".into()))?
            }
            (None, Some(s)) => self
                .repos
                .shared_link
                .get_by_slug(s)
                .await?
                .ok_or_else(|| Error::Unauthorized("invalid share slug".into()))?,
            _ => return Err(Error::BadRequest("missing key or slug".into())),
        };
        if let Some(expires_at) = link.expires_at {
            if expires_at < chrono::Utc::now() {
                return Err(Error::Unauthorized("Shared link expired".into()));
            }
        }
        Ok(AuthContext {
            user_id: link.user_id,
            is_admin: false,
            session_id: None,
            api_key_id: None,
            shared_link_id: Some(link.id),
        })
    }

    pub async fn logout(&self, session_id: Uuid) -> Result<()> {
        self.repos.session.delete(session_id).await
    }

    pub async fn list_api_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        self.repos.api_key.list_for_user(user_id).await
    }

    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        name: &str,
        permissions: &[String],
    ) -> Result<ApiKeyCreateOutcome> {
        let secret = Self::generate_token();
        let api_key = self
            .repos
            .api_key
            .create(user_id, name, &Self::hash_token(&secret), permissions)
            .await?;
        Ok(ApiKeyCreateOutcome { api_key, secret })
    }

    pub async fn delete_api_key(&self, id: Uuid) -> Result<()> {
        self.repos.api_key.delete(id).await
    }

    fn generate_token() -> String {
        use base64::Engine;
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    fn hash_token(token: &str) -> Vec<u8> {
        Sha256::digest(token.as_bytes()).to_vec()
    }
}
