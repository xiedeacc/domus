use super::db_err;
use crate::entities::Session;
use crate::PgPool;
use domus_common::Result;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Look up a session by the SHA-256 digest of the bearer token.
    pub async fn get_by_token_hash(&self, token_hash: &[u8]) -> Result<Option<Session>> {
        sqlx::query_as::<_, Session>(
            r#"SELECT id, lower(hex(token)) AS token, "userId" AS user_id, "deviceType" AS device_type,
                      "deviceOS" AS device_os, "expiresAt" AS expires_at,
                      "createdAt" AS created_at, "updatedAt" AS updated_at
               FROM session WHERE token = $1"#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        token_hash: &[u8],
        device_type: &str,
        device_os: &str,
    ) -> Result<Session> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, Session>(
            r#"INSERT INTO session (id, token, "userId", "deviceType", "deviceOS", "createdAt", "updatedAt")
               VALUES ($1, $2, $3, $4, $5, datetime('now'), datetime('now'))
               RETURNING id, lower(hex(token)) AS token, "userId" AS user_id, "deviceType" AS device_type,
                         "deviceOS" AS device_os, "expiresAt" AS expires_at,
                         "createdAt" AS created_at, "updatedAt" AS updated_at"#,
        )
        .bind(id)
        .bind(token_hash)
        .bind(user_id)
        .bind(device_type)
        .bind(device_os)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Session>> {
        sqlx::query_as::<_, Session>(
            r#"SELECT id, lower(hex(token)) AS token, "userId" AS user_id, "deviceType" AS device_type,
                      "deviceOS" AS device_os, "expiresAt" AS expires_at,
                      "createdAt" AS created_at, "updatedAt" AS updated_at
               FROM session WHERE "userId" = $1 ORDER BY "updatedAt" DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM session WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}
