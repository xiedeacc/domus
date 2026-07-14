use super::db_err;
use crate::entities::ApiKey;
use domus_common::Result;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_key_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        sqlx::query_as::<_, ApiKey>(
            r#"SELECT id, name, key, "userId" AS user_id, permissions,
                      "createdAt" AS created_at, "updatedAt" AS updated_at
               FROM api_key WHERE key = $1"#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        sqlx::query_as::<_, ApiKey>(
            r#"SELECT id, name, key, "userId" AS user_id, permissions,
                      "createdAt" AS created_at, "updatedAt" AS updated_at
               FROM api_key WHERE "userId" = $1 ORDER BY "createdAt" DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        key_hash: &str,
        permissions: &[String],
    ) -> Result<ApiKey> {
        sqlx::query_as::<_, ApiKey>(
            r#"INSERT INTO api_key (name, key, "userId", permissions)
               VALUES ($1, $2, $3, $4)
               RETURNING id, name, key, "userId" AS user_id, permissions,
                         "createdAt" AS created_at, "updatedAt" AS updated_at"#,
        )
        .bind(name)
        .bind(key_hash)
        .bind(user_id)
        .bind(permissions)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM api_key WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}
