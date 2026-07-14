//! Domus server entry point.
//!
//! One binary hosts both worker groups, like Immich's single container:
//!   - api:            REST + socket.io + static web serving
//!   - microservices:  background job workers
//! IMMICH_WORKERS_INCLUDE / DOMUS_WORKERS__* select which groups run, so the
//! deployment can be split into separate processes later without a rebuild.

use domus_api::{build_router, state::AppState, websocket};
use domus_common::Config;
use domus_domain::Services;
use domus_jobs::{handlers::JobContext, PgJobQueue, WorkerPool};
use domus_media::storage::StorageCore;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    let config = Config::load()?;
    info!(port = config.port, media = %config.media_location, "starting domus-server");

    let pool = domus_db::connect(&config).await?;
    let repos = domus_db::Repositories::new(pool.clone());
    let queue = PgJobQueue::new(pool);
    let storage = StorageCore::new(&config.media_location);

    if config.workers.microservices {
        let context = Arc::new(JobContext {
            repos: repos.clone(),
            queue: queue.clone(),
            storage: storage.clone(),
        });
        Arc::new(WorkerPool::new(queue.clone(), context)).start();
    }

    if config.workers.api {
        let services = Services::new(repos, queue, storage);
        let state = AppState::new(config.clone(), services);
        let (socket_layer, _io) = websocket::build();
        let app = build_router(state).layer(socket_layer);

        let addr = format!("{}:{}", config.host, config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!("listening on http://{addr}");
        axum::serve(listener, app).await?;
    } else {
        // Worker-only process: park the main task.
        info!("running in microservices-only mode");
        futures_park().await;
    }

    Ok(())
}

async fn futures_park() {
    let () = std::future::pending().await;
}
