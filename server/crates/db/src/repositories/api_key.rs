use super::db_err;
use crate::entities::ApiKey;
use crate::PgPool;
use domus_common::Result;
use sqlx::Row;
use uuid::Uuid;

#[derive(Clone)]
pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_key_hash(&self, key_hash: &[u8]) -> Result<Option<ApiKey>> {
        let row = sqlx::query(
            r#"SELECT id, name, lower(hex(key)) AS key, "userId" AS user_id, permissions,
                      "createdAt" AS created_at, "updatedAt" AS updated_at
               FROM api_key WHERE key = $1"#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        row.map(|row| api_key_from_row(&row)).transpose()
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        let rows = sqlx::query(
            r#"SELECT id, name, lower(hex(key)) AS key, "userId" AS user_id, permissions,
                      "createdAt" AS created_at, "updatedAt" AS updated_at
               FROM api_key WHERE "userId" = $1 ORDER BY "createdAt" DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        rows.iter().map(api_key_from_row).collect()
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        key_hash: &[u8],
        permissions: &[String],
    ) -> Result<ApiKey> {
        let permissions = serde_json::to_string(permissions)
            .map_err(|e| domus_common::Error::Database(e.to_string()))?;
        let row = sqlx::query(
            r#"INSERT INTO api_key (name, key, "userId", permissions)
               VALUES ($1, $2, $3, $4)
               RETURNING id, name, lower(hex(key)) AS key, "userId" AS user_id, permissions,
                         "createdAt" AS created_at, "updatedAt" AS updated_at"#,
        )
        .bind(name)
        .bind(key_hash)
        .bind(user_id)
        .bind(permissions)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        api_key_from_row(&row)
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

fn api_key_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<ApiKey> {
    let permissions_json: String = row.try_get("permissions").map_err(db_err)?;
    let permissions = serde_json::from_str(&permissions_json).unwrap_or_default();
    Ok(ApiKey {
        id: row.try_get("id").map_err(db_err)?,
        name: row.try_get("name").map_err(db_err)?,
        key: row.try_get("key").map_err(db_err)?,
        user_id: row.try_get("user_id").map_err(db_err)?,
        permissions,
        created_at: row.try_get("created_at").map_err(db_err)?,
        updated_at: row.try_get("updated_at").map_err(db_err)?,
    })
}
