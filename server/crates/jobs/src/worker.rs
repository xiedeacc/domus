//! Worker pool: one polling loop per queue, `default_concurrency()` slots
//! each, dispatching claimed jobs to registered handlers.

use crate::handlers::JobContext;
use crate::queue::{JobData, PgJobQueue};
use crate::types::QueueName;
use domus_common::Result;
use std::sync::Arc;
use tracing::{error, info};

pub struct WorkerPool {
    queue: PgJobQueue,
    context: Arc<JobContext>,
}

impl WorkerPool {
    pub fn new(queue: PgJobQueue, context: Arc<JobContext>) -> Self {
        Self { queue, context }
    }

    /// Spawn the polling loops. Returns immediately; loops run until the
    /// process shuts down.
    pub fn start(self: Arc<Self>) {
        for &queue_name in QueueName::all() {
            for slot in 0..queue_name.default_concurrency() {
                let this = Arc::clone(&self);
                tokio::spawn(async move {
                    this.run_loop(queue_name, slot).await;
                });
            }
        }
        info!("worker pool started");
    }

    async fn run_loop(&self, queue: QueueName, _slot: usize) {
        loop {
            match self.queue.claim(queue).await {
                Ok(Some(job)) => {
                    let id = job.id;
                    match self.dispatch(job).await {
                        Ok(()) => {
                            let _ = self.queue.complete(id).await;
                        }
                        Err(e) => {
                            error!(?queue, job = %id, "job failed: {e}");
                            let _ = self.queue.fail(id, &e.to_string()).await;
                        }
                    }
                }
                Ok(None) => {
                    // Queue drained — sleep before polling again. A LISTEN/
                    // NOTIFY wakeup replaces this in the full implementation.
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                Err(_) => {
                    // NotImplemented during scaffolding, or DB hiccup: back off.
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    async fn dispatch(&self, job: JobData) -> Result<()> {
        crate::handlers::handle(&self.context, job).await
    }
}
