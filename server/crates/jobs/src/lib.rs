//! Background job system.
//!
//! Immich runs BullMQ on Redis; Domus replaces it with a PostgreSQL-backed
//! queue (`FOR UPDATE SKIP LOCKED`) so a deployment needs no Redis. The queue
//! is internal — nothing about it is part of the client protocol — only the
//! /jobs and /queues admin APIs surface its state, and those responses stay
//! Immich-shaped.

pub mod handlers;
pub mod queue;
pub mod types;
pub mod worker;

pub use queue::{JobData, PgJobQueue};
pub use types::{JobName, QueueName};
pub use worker::WorkerPool;
