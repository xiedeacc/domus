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
use domus_db::entities::{Session, User};
use domus_db::Repositories;
use sha2::{Digest, Sha256};
use uuid::Uuid;

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

impl AuthService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn login(&self, email: &str, password: &str, device: (&str, &str)) -> Result<LoginOutcome> {
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
        let token = Self::generate_token();
        let session = self
            .repos
            .session
            .create(user.id, &Self::hash_token(&token), device.0, device.1)
            .await?;
        Ok(LoginOutcome { user, token, session })
    }

    /// First user to register becomes the admin (Immich's onboarding rule).
    pub async fn admin_sign_up(&self, email: &str, password: &str, name: &str) -> Result<User> {
        if self.repos.user.count_admins().await? > 0 {
            return Err(Error::BadRequest("The server already has an admin".into()));
        }
        let hash = bcrypt::hash(password, 10).map_err(|e| Error::Internal(e.into()))?;
        self.repos.user.create(email, &hash, name, true).await
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

    pub async fn logout(&self, session_id: Uuid) -> Result<()> {
        self.repos.session.delete(session_id).await
    }

    fn generate_token() -> String {
        use base64::Engine;
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    fn hash_token(token: &str) -> String {
        hex::encode(Sha256::digest(token.as_bytes()))
    }
}
