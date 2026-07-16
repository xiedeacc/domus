use super::db_err;
use crate::entities::Memory;
use crate::PgPool;
use domus_common::{Error, Result};
use sqlx::Row;
use uuid::Uuid;

#[derive(Clone)]
pub struct MemoryRepository {
    pool: PgPool,
}

impl MemoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Memory>> {
        let rows = sqlx::query(MEMORY_LIST_SQL)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?;
        rows.iter().map(memory_from_row).collect()
    }

    pub async fn get(&self, id: Uuid) -> Result<Memory> {
        let row = sqlx::query(MEMORY_GET_SQL)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(db_err)?;
        memory_from_row(&row)
    }

    pub async fn create_on_this_day(&self, user_id: Uuid) -> Result<Vec<Memory>> {
        self.list_for_user(user_id).await
    }

    pub async fn asset_ids(&self, memory_id: Uuid) -> Result<Vec<Uuid>> {
        sqlx::query_scalar(
            r#"SELECT "assetId" FROM memory_asset WHERE "memoryId" = $1 ORDER BY "assetId""#,
        )
        .bind(memory_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)
    }
}

const MEMORY_LIST_SQL: &str = r#"SELECT id, "ownerId", type, data, "memoryAt", "isSaved",
       "seenAt", "createdAt", "updatedAt", "deletedAt"
FROM memory
WHERE "ownerId" = $1 AND "deletedAt" IS NULL
ORDER BY "memoryAt" DESC"#;

const MEMORY_GET_SQL: &str = r#"SELECT id, "ownerId", type, data, "memoryAt", "isSaved",
       "seenAt", "createdAt", "updatedAt", "deletedAt"
FROM memory
WHERE id = $1"#;

fn memory_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Memory> {
    let data: String = row.try_get("data").map_err(db_err)?;
    let data = serde_json::from_str(&data).map_err(|e| Error::Database(e.to_string()))?;
    Ok(Memory {
        id: row.try_get("id").map_err(db_err)?,
        owner_id: row.try_get("ownerId").map_err(db_err)?,
        memory_type: row.try_get("type").map_err(db_err)?,
        data,
        memory_at: row.try_get("memoryAt").map_err(db_err)?,
        is_saved: row.try_get("isSaved").map_err(db_err)?,
        seen_at: row.try_get("seenAt").map_err(db_err)?,
        created_at: row.try_get("createdAt").map_err(db_err)?,
        updated_at: row.try_get("updatedAt").map_err(db_err)?,
        deleted_at: row.try_get("deletedAt").map_err(db_err)?,
    })
}
