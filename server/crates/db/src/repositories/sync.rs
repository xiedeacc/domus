use super::db_err;
use crate::PgPool;
use domus_common::Result;
use uuid::Uuid;

#[derive(Clone)]
pub struct SyncRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl SyncRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_checkpoints(&self, session_id: Uuid) -> Result<Vec<(String, String)>> {
        sqlx::query_as::<_, (String, String)>(
            r#"SELECT "type", "ack"
               FROM session_sync_checkpoint
               WHERE "sessionId" = $1
               ORDER BY "type""#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn upsert_checkpoints(&self, session_id: Uuid, acks: &[String]) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for ack in acks {
            let Some((entity_type, _)) = ack.split_once(':') else {
                continue;
            };
            sqlx::query(
                r#"INSERT INTO session_sync_checkpoint ("sessionId", "type", "ack", "createdAt", "updatedAt")
                   VALUES ($1, $2, $3, datetime('now'), datetime('now'))
                   ON CONFLICT ("sessionId", "type") DO UPDATE
                   SET "ack" = excluded."ack", "updatedAt" = datetime('now')"#,
            )
            .bind(session_id)
            .bind(entity_type)
            .bind(ack)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }

    pub async fn delete_checkpoints(&self, session_id: Uuid, types: &[String]) -> Result<()> {
        if types.is_empty() {
            sqlx::query(r#"DELETE FROM session_sync_checkpoint WHERE "sessionId" = $1"#)
                .bind(session_id)
                .execute(&self.pool)
                .await
                .map_err(db_err)?;
            return Ok(());
        }

        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for entity_type in types {
            sqlx::query(
                r#"DELETE FROM session_sync_checkpoint WHERE "sessionId" = $1 AND "type" = $2"#,
            )
            .bind(session_id)
            .bind(entity_type)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }
}
