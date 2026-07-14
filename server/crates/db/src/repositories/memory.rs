use super::db_err;
use crate::entities::Memory;
use domus_common::Result;
use sqlx::{PgPool, Row};
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
        let rows = sqlx::query(
            r#"SELECT EXTRACT(YEAR FROM "localDateTime")::int AS year, array_agg(id ORDER BY "localDateTime" DESC) AS asset_ids
               FROM asset
               WHERE "ownerId" = $1
                 AND "deletedAt" IS NULL
                 AND EXTRACT(MONTH FROM "localDateTime") = EXTRACT(MONTH FROM now())
                 AND EXTRACT(DAY FROM "localDateTime") = EXTRACT(DAY FROM now())
                 AND EXTRACT(YEAR FROM "localDateTime") < EXTRACT(YEAR FROM now())
               GROUP BY year
               ORDER BY year DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;

        let mut memories = Vec::new();
        for row in rows {
            let year: i32 = row.try_get("year").map_err(db_err)?;
            let asset_ids: Vec<Uuid> = row.try_get("asset_ids").map_err(db_err)?;
            let data = serde_json::json!({
                "year": year,
                "title": format!("{} years ago", chrono::Utc::now().format("%Y").to_string().parse::<i32>().unwrap_or(year) - year),
            });
            let mut tx = self.pool.begin().await.map_err(db_err)?;
            let existing = sqlx::query(
                r#"SELECT id, "ownerId", type, data, "memoryAt", "isSaved", "seenAt",
                          "createdAt", "updatedAt", "deletedAt"
                   FROM memory
                   WHERE "ownerId" = $1 AND type = 'on_this_day' AND data->>'year' = $2
                     AND "deletedAt" IS NULL
                   LIMIT 1"#,
            )
            .bind(user_id)
            .bind(year.to_string())
            .fetch_optional(&mut *tx)
            .await
            .map_err(db_err)?;

            let memory = if let Some(row) = existing {
                memory_from_row(&row)?
            } else {
                let row = sqlx::query(
                    r#"INSERT INTO memory ("ownerId", type, data, "memoryAt")
                       VALUES ($1, 'on_this_day', $2, now())
                       RETURNING id, "ownerId", type, data, "memoryAt", "isSaved", "seenAt",
                                 "createdAt", "updatedAt", "deletedAt""#,
                )
                .bind(user_id)
                .bind(data)
                .fetch_one(&mut *tx)
                .await
                .map_err(db_err)?;
                memory_from_row(&row)?
            };

            for asset_id in asset_ids {
                sqlx::query(
                    r#"INSERT INTO memory_asset ("memoryId", "assetId")
                       VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
                )
                .bind(memory.id)
                .bind(asset_id)
                .execute(&mut *tx)
                .await
                .map_err(db_err)?;
            }
            tx.commit().await.map_err(db_err)?;
            memories.push(memory);
        }
        Ok(memories)
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

fn memory_from_row(row: &sqlx::postgres::PgRow) -> Result<Memory> {
    Ok(Memory {
        id: row.try_get("id").map_err(db_err)?,
        owner_id: row.try_get("ownerId").map_err(db_err)?,
        memory_type: row.try_get("type").map_err(db_err)?,
        data: row.try_get("data").map_err(db_err)?,
        memory_at: row.try_get("memoryAt").map_err(db_err)?,
        is_saved: row.try_get("isSaved").map_err(db_err)?,
        seen_at: row.try_get("seenAt").map_err(db_err)?,
        created_at: row.try_get("createdAt").map_err(db_err)?,
        updated_at: row.try_get("updatedAt").map_err(db_err)?,
        deleted_at: row.try_get("deletedAt").map_err(db_err)?,
    })
}
