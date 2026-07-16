use super::db_err;
use crate::PgPool;
use domus_common::{Error, Result};

#[derive(Clone)]
pub struct SystemMetadataRepository {
    pool: PgPool,
}

impl SystemMetadataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let value: Option<String> =
            sqlx::query_scalar(r#"SELECT value FROM system_metadata WHERE key = $1"#)
                .bind(key)
                .fetch_optional(&self.pool)
                .await
                .map_err(db_err)?;
        value
            .map(|value| serde_json::from_str(&value).map_err(|e| Error::Database(e.to_string())))
            .transpose()
    }

    pub async fn set(&self, key: &str, value: serde_json::Value) -> Result<()> {
        let value = serde_json::to_string(&value).map_err(|e| Error::Database(e.to_string()))?;
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
