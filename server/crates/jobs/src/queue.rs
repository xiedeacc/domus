//! PostgreSQL-backed job queue. Claims use `FOR UPDATE SKIP LOCKED` so any
//! number of worker processes can share one table safely.

use crate::types::{JobName, QueueName};
use domus_common::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    pub async fn enqueue(&self, _name: JobName, _payload: serde_json::Value) -> Result<Uuid> {
        // TODO: INSERT INTO jobs (queue, name, payload, status) VALUES (...)
        //       and NOTIFY the queue channel.
        Err(Error::NotImplemented("PgJobQueue::enqueue"))
    }

    /// Claim the next pending job on `queue`, or None if the queue is empty.
    pub async fn claim(&self, _queue: QueueName) -> Result<Option<JobData>> {
        // TODO: UPDATE jobs SET status='active' WHERE id = (
        //         SELECT id FROM jobs WHERE queue=$1 AND status='waiting'
        //         ORDER BY "createdAt" LIMIT 1 FOR UPDATE SKIP LOCKED)
        //       RETURNING ...
        Err(Error::NotImplemented("PgJobQueue::claim"))
    }

    pub async fn complete(&self, _id: Uuid) -> Result<()> {
        Err(Error::NotImplemented("PgJobQueue::complete"))
    }

    /// Mark failed; re-queue with backoff until max attempts is reached.
    pub async fn fail(&self, _id: Uuid, _error: &str) -> Result<()> {
        Err(Error::NotImplemented("PgJobQueue::fail"))
    }

    /// waiting/active/completed/failed/delayed/paused counts for the
    /// /queues admin API.
    pub async fn counts(&self, _queue: QueueName) -> Result<serde_json::Value> {
        Err(Error::NotImplemented("PgJobQueue::counts"))
    }

    pub async fn pause(&self, _queue: QueueName, _paused: bool) -> Result<()> {
        Err(Error::NotImplemented("PgJobQueue::pause"))
    }

    pub async fn clear(&self, _queue: QueueName, _failed_only: bool) -> Result<()> {
        Err(Error::NotImplemented("PgJobQueue::clear"))
    }
}
