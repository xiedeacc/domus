//! Data access layer: SQLite pool, migrations and one repository module per
//! aggregate. Repositories expose typed methods; SQL lives here only.

pub mod entities;
pub mod repositories;

use domus_common::{Config, Error, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
pub use sqlx::SqlitePool as PgPool;
use std::str::FromStr;

/// Connect to SQLite and run pending migrations when explicitly requested.
pub async fn connect(config: &Config) -> Result<PgPool> {
    let options = SqliteConnectOptions::from_str(&config.database.url)
        .map_err(|e| Error::Database(e.to_string()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(false);
    let pool = SqlitePoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect_with(options)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    if config.database.run_migrations {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
    } else {
        ensure_runtime_tables(&pool).await?;
    }
    Ok(pool)
}

async fn ensure_runtime_tables(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "job" (
            "id" text PRIMARY KEY DEFAULT (
                lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' ||
                substr(lower(hex(randomblob(2))), 2) || '-' ||
                substr('89ab', abs(random()) % 4 + 1, 1) ||
                substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6)))
            ),
            "queue" varchar NOT NULL,
            "name" varchar NOT NULL,
            "payload" text NOT NULL DEFAULT '{}',
            "status" varchar NOT NULL DEFAULT 'waiting',
            "attempts" integer NOT NULL DEFAULT 0,
            "maxAttempts" integer NOT NULL DEFAULT 3,
            "error" text,
            "runAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "createdAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "updatedAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::Database(e.to_string()))?;
    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS "IDX_job_claim" ON "job" ("queue", "status", "runAt")"#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::Database(e.to_string()))?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "asset_job_status" (
            "assetId" text PRIMARY KEY,
            "metadataExtractedAt" text,
            "previewAt" text,
            "thumbnailAt" text
        )"#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::Database(e.to_string()))?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "tag" (
            "id" blob PRIMARY KEY,
            "userId" blob NOT NULL,
            "value" text NOT NULL,
            "color" text,
            "parentId" blob,
            "createdAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "updatedAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::Database(e.to_string()))?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "tag_asset" (
            "tagId" blob NOT NULL,
            "assetId" blob NOT NULL,
            PRIMARY KEY ("tagId", "assetId")
        )"#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::Database(e.to_string()))?;
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "session_sync_checkpoint" (
            "sessionId" blob NOT NULL,
            "type" text NOT NULL,
            "ack" text NOT NULL,
            "createdAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "updatedAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY ("sessionId", "type")
        )"#,
    )
    .execute(pool)
    .await
    .map_err(|e| Error::Database(e.to_string()))?;
    ensure_ml_tables(pool).await?;
    Ok(())
}

async fn ensure_ml_tables(pool: &PgPool) -> Result<()> {
    for sql in [
        r#"CREATE TABLE IF NOT EXISTS "smart_search" (
            "assetId" blob PRIMARY KEY,
            "embedding" text NOT NULL
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "person" (
            "id" blob PRIMARY KEY,
            "createdAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "updatedAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "ownerId" blob NOT NULL,
            "name" text NOT NULL DEFAULT '',
            "thumbnailPath" text NOT NULL DEFAULT '',
            "isHidden" integer NOT NULL DEFAULT 0,
            "birthDate" text,
            "faceAssetId" blob,
            "isFavorite" integer NOT NULL DEFAULT 0,
            "color" text,
            "updateId" blob NOT NULL
        )"#,
        r#"CREATE INDEX IF NOT EXISTS "IDX_person_owner" ON "person" ("ownerId")"#,
        r#"CREATE TABLE IF NOT EXISTS "asset_face" (
            "id" blob PRIMARY KEY,
            "assetId" blob NOT NULL,
            "personId" blob,
            "imageWidth" integer NOT NULL DEFAULT 0,
            "imageHeight" integer NOT NULL DEFAULT 0,
            "boundingBoxX1" integer NOT NULL DEFAULT 0,
            "boundingBoxY1" integer NOT NULL DEFAULT 0,
            "boundingBoxX2" integer NOT NULL DEFAULT 0,
            "boundingBoxY2" integer NOT NULL DEFAULT 0,
            "sourceType" text NOT NULL DEFAULT 'machine-learning',
            "deletedAt" text,
            "updatedAt" text NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "updateId" blob NOT NULL,
            "isVisible" integer NOT NULL DEFAULT 1
        )"#,
        r#"CREATE INDEX IF NOT EXISTS "asset_face_assetId_personId_idx" ON "asset_face" ("assetId", "personId")"#,
        r#"CREATE INDEX IF NOT EXISTS "asset_face_personId_assetId_idx" ON "asset_face" ("personId", "assetId")"#,
        r#"CREATE TABLE IF NOT EXISTS "face_search" (
            "faceId" blob PRIMARY KEY,
            "embedding" text NOT NULL
        )"#,
        r#"CREATE TABLE IF NOT EXISTS "asset_ocr" (
            "id" blob PRIMARY KEY,
            "assetId" blob NOT NULL,
            "x1" real NOT NULL,
            "y1" real NOT NULL,
            "x2" real NOT NULL,
            "y2" real NOT NULL,
            "x3" real NOT NULL,
            "y3" real NOT NULL,
            "x4" real NOT NULL,
            "y4" real NOT NULL,
            "boxScore" real NOT NULL,
            "textScore" real NOT NULL,
            "text" text NOT NULL,
            "isVisible" integer NOT NULL DEFAULT 1,
            "updateId" blob NOT NULL
        )"#,
        r#"CREATE INDEX IF NOT EXISTS "IDX_asset_ocr_assetId" ON "asset_ocr" ("assetId")"#,
        r#"CREATE TABLE IF NOT EXISTS "ocr_search" (
            "assetId" blob PRIMARY KEY,
            "text" text NOT NULL
        )"#,
    ] {
        sqlx::query(sql)
            .execute(pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
    }
    Ok(())
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
