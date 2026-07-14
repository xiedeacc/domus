//! Job handlers — the asset ingestion pipeline lives here.
//!
//! Upload flow (mirrors Immich):
//!   upload → MetadataExtraction → GeneratePreview → GenerateThumbnail
//!          → GenerateThumbhash → (video) VideoConversion
//!          → StorageTemplateMigration (if enabled)

use crate::queue::{JobData, PgJobQueue};
use crate::types::JobName;
use domus_common::{Error, Result};
use domus_db::Repositories;
use domus_media::storage::StorageCore;

/// Everything a job handler may need.
pub struct JobContext {
    pub repos: Repositories,
    pub queue: PgJobQueue,
    pub storage: StorageCore,
}

pub async fn handle(ctx: &JobContext, job: JobData) -> Result<()> {
    match job.name {
        JobName::MetadataExtraction => metadata_extraction(ctx, &job).await,
        JobName::GeneratePreview => generate_preview(ctx, &job).await,
        JobName::GenerateThumbnail => generate_thumbnail(ctx, &job).await,
        JobName::GenerateThumbhash => generate_thumbhash(ctx, &job).await,
        JobName::VideoConversion => video_conversion(ctx, &job).await,
        JobName::AssetDeletion => asset_deletion(ctx, &job).await,
        _ => Err(Error::NotImplemented("job handler")),
    }
}

/// exiftool pass: dimensions, timestamps, GPS → exif row; reverse geocode;
/// then fan out thumbnail jobs.
async fn metadata_extraction(_ctx: &JobContext, _job: &JobData) -> Result<()> {
    Err(Error::NotImplemented("handlers::metadata_extraction"))
}

async fn generate_preview(_ctx: &JobContext, _job: &JobData) -> Result<()> {
    Err(Error::NotImplemented("handlers::generate_preview"))
}

async fn generate_thumbnail(_ctx: &JobContext, _job: &JobData) -> Result<()> {
    Err(Error::NotImplemented("handlers::generate_thumbnail"))
}

async fn generate_thumbhash(_ctx: &JobContext, _job: &JobData) -> Result<()> {
    Err(Error::NotImplemented("handlers::generate_thumbhash"))
}

async fn video_conversion(_ctx: &JobContext, _job: &JobData) -> Result<()> {
    Err(Error::NotImplemented("handlers::video_conversion"))
}

/// Permanently remove a trashed asset: delete files, exif, then the row.
async fn asset_deletion(_ctx: &JobContext, _job: &JobData) -> Result<()> {
    Err(Error::NotImplemented("handlers::asset_deletion"))
}
