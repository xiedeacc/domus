//! SQLite-backed job queue for Domus' single-binary deployment.

use crate::types::{JobName, QueueName};
use domus_common::{Error, Result};
use domus_db::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobData {
    pub id: Uuid,
    pub name: JobName,
    pub payload: serde_json::Value,
    pub attempts: i32,
}

#[derive(Clone)]
pub struct PgJobQueue {
    #[allow(dead_code)]
    pool: PgPool,
}

impl PgJobQueue {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert a job. `payload` is typically `{"id": "<assetId>"}`.
    pub async fn enqueue(&self, name: JobName, payload: serde_json::Value) -> Result<Uuid> {
        let queue = queue_name(name.queue())?;
        let name_value = job_name(&name)?;
        let (id,): (Uuid,) = sqlx::query_as(
            r#"INSERT INTO job (queue, name, payload, status)
               VALUES ($1, $2, $3, 'waiting')
               RETURNING id"#,
        )
        .bind(queue)
        .bind(name_value)
        .bind(payload)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(id)
    }

    /// Claim the next pending job on `queue`, or None if the queue is empty.
    pub async fn claim(&self, queue: QueueName) -> Result<Option<JobData>> {
        let queue = queue_name(queue)?;
        let row = sqlx::query_as::<_, JobRow>(
            r#"UPDATE job
               SET status = 'active', attempts = attempts + 1, "updatedAt" = datetime('now')
               WHERE id = (
                   SELECT id FROM job
                   WHERE queue = $1 AND status = 'waiting' AND "runAt" <= datetime('now')
                   ORDER BY "createdAt"
                   LIMIT 1
               )
               RETURNING id, name, payload, attempts"#,
        )
        .bind(queue)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?;
        row.map(TryInto::try_into).transpose()
    }

    pub async fn complete(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            r#"UPDATE job SET status = 'completed', "updatedAt" = datetime('now') WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    /// Mark failed; re-queue with backoff until max attempts is reached.
    pub async fn fail(&self, id: Uuid, error: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE job
               SET status = CASE WHEN attempts >= "maxAttempts" THEN 'failed' ELSE 'waiting' END,
                   error = $2,
                   "runAt" = CASE
                       WHEN attempts >= "maxAttempts" THEN "runAt"
                       ELSE datetime('now', '+' || min(300, max(5, attempts * attempts * 5)) || ' seconds')
                   END,
                   "updatedAt" = datetime('now')
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    /// waiting/active/completed/failed/delayed/paused counts for the
    /// /queues admin API.
    pub async fn counts(&self, queue: QueueName) -> Result<serde_json::Value> {
        let queue = queue_name(queue)?;
        let rows: Vec<(String, i64)> =
            sqlx::query_as(r#"SELECT status, COUNT(*) FROM job WHERE queue = $1 GROUP BY status"#)
                .bind(queue)
                .fetch_all(&self.pool)
                .await
                .map_err(db_err)?;
        let mut value = serde_json::json!({
            "waiting": 0,
            "active": 0,
            "completed": 0,
            "failed": 0,
            "delayed": 0,
            "paused": 0
        });
        for (status, count) in rows {
            value[status] = serde_json::json!(count);
        }
        Ok(value)
    }

    pub async fn pause(&self, queue: QueueName, paused: bool) -> Result<()> {
        let queue = queue_name(queue)?;
        let (from, to) = if paused {
            ("waiting", "paused")
        } else {
            ("paused", "waiting")
        };
        sqlx::query(
            r#"UPDATE job SET status = $1, "updatedAt" = datetime('now') WHERE queue = $2 AND status = $3"#,
        )
        .bind(to)
        .bind(queue)
        .bind(from)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    pub async fn clear(&self, queue: QueueName, failed_only: bool) -> Result<()> {
        let queue = queue_name(queue)?;
        let statuses: &[&str] = if failed_only {
            &["failed"]
        } else {
            &["waiting", "completed", "failed", "delayed", "paused"]
        };
        for status in statuses {
            sqlx::query(r#"DELETE FROM job WHERE queue = $1 AND status = $2"#)
                .bind(&queue)
                .bind(status)
                .execute(&self.pool)
                .await
                .map_err(db_err)?;
        }
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct JobRow {
    id: Uuid,
    name: String,
    payload: serde_json::Value,
    attempts: i32,
}

impl TryFrom<JobRow> for JobData {
    type Error = Error;

    fn try_from(row: JobRow) -> Result<Self> {
        Ok(Self {
            id: row.id,
            name: serde_json::from_value(serde_json::Value::String(row.name))
                .map_err(|e| Error::Database(e.to_string()))?,
            payload: row.payload,
            attempts: row.attempts,
        })
    }
}

fn queue_name(queue: QueueName) -> Result<String> {
    serde_json::to_value(queue)
        .and_then(serde_json::from_value)
        .map_err(|e| Error::Database(e.to_string()))
}

fn job_name(name: &JobName) -> Result<String> {
    serde_json::to_value(name)
        .and_then(serde_json::from_value)
        .map_err(|e| Error::Database(e.to_string()))
}

fn db_err(e: sqlx::Error) -> Error {
    Error::Database(e.to_string())
}
