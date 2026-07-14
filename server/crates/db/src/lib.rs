//! Data access layer: PostgreSQL pool, migrations and one repository module
//! per aggregate. Repositories expose typed methods; SQL lives here only.

pub mod entities;
pub mod repositories;

use domus_common::{Config, Error, Result};
use sqlx::postgres::PgPoolOptions;
pub use sqlx::PgPool;

/// Connect to PostgreSQL and run pending migrations.
pub async fn connect(config: &Config) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(pool)
}

/// Bundle of all repositories, cloned cheaply (each holds the shared pool).
#[derive(Clone)]
pub struct Repositories {
    pub user: repositories::user::UserRepository,
    pub session: repositories::session::SessionRepository,
    pub api_key: repositories::api_key::ApiKeyRepository,
    pub asset: repositories::asset::AssetRepository,
    pub album: repositories::album::AlbumRepository,
    pub memory: repositories::memory::MemoryRepository,
    pub partner: repositories::partner::PartnerRepository,
    pub shared_link: repositories::shared_link::SharedLinkRepository,
    pub tag: repositories::tag::TagRepository,
    pub stack: repositories::stack::StackRepository,
    pub library: repositories::library::LibraryRepository,
    pub activity: repositories::activity::ActivityRepository,
    pub notification: repositories::notification::NotificationRepository,
    pub timeline: repositories::timeline::TimelineRepository,
    pub search: repositories::search::SearchRepository,
    pub sync: repositories::sync::SyncRepository,
    pub job: repositories::job::JobRepository,
    pub system_metadata: repositories::system_metadata::SystemMetadataRepository,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user: repositories::user::UserRepository::new(pool.clone()),
            session: repositories::session::SessionRepository::new(pool.clone()),
            api_key: repositories::api_key::ApiKeyRepository::new(pool.clone()),
            asset: repositories::asset::AssetRepository::new(pool.clone()),
            album: repositories::album::AlbumRepository::new(pool.clone()),
            memory: repositories::memory::MemoryRepository::new(pool.clone()),
            partner: repositories::partner::PartnerRepository::new(pool.clone()),
            shared_link: repositories::shared_link::SharedLinkRepository::new(pool.clone()),
            tag: repositories::tag::TagRepository::new(pool.clone()),
            stack: repositories::stack::StackRepository::new(pool.clone()),
            library: repositories::library::LibraryRepository::new(pool.clone()),
            activity: repositories::activity::ActivityRepository::new(pool.clone()),
            notification: repositories::notification::NotificationRepository::new(pool.clone()),
            timeline: repositories::timeline::TimelineRepository::new(pool.clone()),
            search: repositories::search::SearchRepository::new(pool.clone()),
            sync: repositories::sync::SyncRepository::new(pool.clone()),
            job: repositories::job::JobRepository::new(pool.clone()),
            system_metadata: repositories::system_metadata::SystemMetadataRepository::new(pool),
        }
    }
}
