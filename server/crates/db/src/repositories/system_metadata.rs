use super::db_err;
use domus_common::Result;
use sqlx::PgPool;

#[derive(Clone)]
pub struct SystemMetadataRepository {
    pool: PgPool,
}

impl SystemMetadataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        sqlx::query_scalar(r#"SELECT value FROM system_metadata WHERE key = $1"#)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)
    }

    pub async fn set(&self, key: &str, value: serde_json::Value) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO system_metadata (key, value)
               VALUES ($1, $2)
               ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }
}
