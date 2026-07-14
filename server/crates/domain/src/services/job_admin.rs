//! Admin job/queue control (GET/PUT /jobs, /queues APIs).

use domus_common::{Error, Result};
use domus_jobs::{PgJobQueue, QueueName};

pub struct JobAdminService {
    queue: PgJobQueue,
}

impl JobAdminService {
    pub fn new(queue: PgJobQueue) -> Self {
        Self { queue }
    }

    /// Status of every queue, keyed by queue name (JobStatusDto shape).
    pub async fn all_status(&self) -> Result<serde_json::Value> {
        let mut map = serde_json::Map::new();
        for &q in QueueName::all() {
            let counts = match self.queue.counts(q).await {
                Ok(c) => c,
                Err(_) => serde_json::json!({
                    "active": 0, "completed": 0, "failed": 0,
                    "delayed": 0, "waiting": 0, "paused": 0
                }),
            };
            map.insert(
                serde_json::to_value(q)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
                serde_json::json!({
                    "jobCounts": counts,
                    "queueStatus": { "isActive": false, "isPaused": false },
                }),
            );
        }
        Ok(serde_json::Value::Object(map))
    }

    /// Handle a queue command: start / pause / resume / empty / clear-failed.
    pub async fn command(&self, queue: QueueName, command: &str) -> Result<()> {
        match command {
            "pause" => self.queue.pause(queue, true).await,
            "resume" => self.queue.pause(queue, false).await,
            "empty" => self.queue.clear(queue, false).await,
            "clear-failed" => self.queue.clear(queue, true).await,
            "start" => Err(Error::NotImplemented("queue start commands")),
            other => Err(Error::BadRequest(format!("unknown command: {other}"))),
        }
    }
}
