//! Job handlers — the asset ingestion pipeline lives here.
//!
//! Upload flow (mirrors Immich):
//!   upload → MetadataExtraction → GeneratePreview → GenerateThumbnail
//!          → GenerateThumbhash → (video) VideoConversion
//!          → StorageTemplateMigration (if enabled)

use crate::queue::{JobData, PgJobQueue};
use crate::types::JobName;
use domus_common::types::AssetType;
use domus_common::{Error, Result};
use domus_db::entities::Exif;
use domus_db::Repositories;
use domus_media::storage::StorageCore;
use domus_media::{exif, thumbnail, transcode};
use std::path::{Path, PathBuf};
use uuid::Uuid;

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
        JobName::SmartSearch => smart_search(ctx, &job).await,
        JobName::AssetDeletion => asset_deletion(ctx, &job).await,
        _ => Err(Error::NotImplemented("job handler")),
    }
}

/// exiftool pass: dimensions, timestamps, GPS → exif row; reverse geocode;
/// then fan out thumbnail jobs.
async fn metadata_extraction(ctx: &JobContext, job: &JobData) -> Result<()> {
    let asset_id = job_asset_id(job)?;
    let asset = ctx.repos.asset.get(asset_id).await?;
    let original = PathBuf::from(&asset.original_path);
    let file_size = tokio::fs::metadata(&original)
        .await
        .ok()
        .map(|m| m.len() as i64);
    let mut exif_row = Exif {
        asset_id,
        file_size_in_byte: file_size,
        ..Default::default()
    };

    if asset.asset_type == AssetType::Video {
        if let Ok(info) = transcode::probe(&original).await {
            exif_row.exif_image_width = (info.width > 0).then_some(info.width);
            exif_row.exif_image_height = (info.height > 0).then_some(info.height);
            exif_row.fps = info.fps;
            let duration = duration_millis(info.duration_seconds);
            ctx.repos
                .asset
                .update_duration(asset_id, Some(duration))
                .await?;
        }
    } else if let Ok(data) = exif::extract(&original).await {
        exif_row.make = data.make;
        exif_row.model = data.model;
        exif_row.exif_image_width = data.image_width;
        exif_row.exif_image_height = data.image_height;
        exif_row.date_time_original = data
            .date_time_original
            .as_deref()
            .and_then(parse_exif_datetime);
        exif_row.time_zone = data.offset_time_original;
        exif_row.latitude = data.gps_latitude;
        exif_row.longitude = data.gps_longitude;
        exif_row.orientation = data.orientation.map(|v| v.to_string());
        exif_row.iso = data.iso;
        exif_row.f_number = data.f_number;
        exif_row.focal_length = data.focal_length;
        exif_row.exposure_time = data.exposure_time;
        exif_row.lens_model = data.lens_model;
        exif_row.rating = data.rating;
    }

    ctx.repos.asset.upsert_exif(&exif_row).await?;
    ctx.repos.asset.mark_metadata_extracted(asset_id).await?;
    ctx.queue
        .enqueue(
            JobName::GeneratePreview,
            serde_json::json!({ "id": asset_id }),
        )
        .await?;
    ctx.queue
        .enqueue(
            JobName::GenerateThumbnail,
            serde_json::json!({ "id": asset_id }),
        )
        .await?;
    if asset.asset_type == AssetType::Video {
        ctx.queue
            .enqueue(
                JobName::VideoConversion,
                serde_json::json!({ "id": asset_id }),
            )
            .await?;
    }
    ctx.queue
        .enqueue(JobName::SmartSearch, serde_json::json!({ "id": asset_id }))
        .await?;
    Ok(())
}

async fn generate_preview(ctx: &JobContext, job: &JobData) -> Result<()> {
    let asset = ctx.repos.asset.get(job_asset_id(job)?).await?;
    let output = ctx.storage.preview_path(asset.owner_id, asset.id);
    thumbnail::generate(
        Path::new(&asset.original_path),
        &output,
        thumbnail::ThumbnailOptions::preview(),
    )
    .await?;
    ctx.repos
        .asset
        .upsert_asset_file(asset.id, "preview", &output.to_string_lossy())
        .await?;
    ctx.repos.asset.mark_preview_generated(asset.id).await?;
    Ok(())
}

async fn generate_thumbnail(ctx: &JobContext, job: &JobData) -> Result<()> {
    let asset = ctx.repos.asset.get(job_asset_id(job)?).await?;
    let output = ctx.storage.thumbnail_path(asset.owner_id, asset.id);
    thumbnail::generate(
        Path::new(&asset.original_path),
        &output,
        thumbnail::ThumbnailOptions::thumbnail(),
    )
    .await?;
    ctx.repos
        .asset
        .upsert_asset_file(asset.id, "thumbnail", &output.to_string_lossy())
        .await?;
    ctx.repos.asset.mark_thumbnail_generated(asset.id).await?;
    ctx.queue
        .enqueue(
            JobName::GenerateThumbhash,
            serde_json::json!({ "id": asset.id }),
        )
        .await?;
    Ok(())
}

async fn generate_thumbhash(ctx: &JobContext, job: &JobData) -> Result<()> {
    let asset = ctx.repos.asset.get(job_asset_id(job)?).await?;
    let thumbnail_path = ctx.storage.thumbnail_path(asset.owner_id, asset.id);
    let hash = thumbnail::thumbhash(&thumbnail_path).await?;
    ctx.repos.asset.update_thumbhash(asset.id, &hash).await?;
    Ok(())
}

async fn video_conversion(ctx: &JobContext, job: &JobData) -> Result<()> {
    let asset = ctx.repos.asset.get(job_asset_id(job)?).await?;
    if asset.asset_type != AssetType::Video {
        return Ok(());
    }
    let output = ctx.storage.encoded_video_path(asset.owner_id, asset.id);
    transcode::transcode(
        Path::new(&asset.original_path),
        &output,
        transcode::TranscodeOptions::default(),
    )
    .await?;
    ctx.repos
        .asset
        .upsert_asset_file(asset.id, "encoded-video", &output.to_string_lossy())
        .await?;
    Ok(())
}

async fn smart_search(ctx: &JobContext, job: &JobData) -> Result<()> {
    let asset = ctx.repos.asset.get(job_asset_id(job)?).await?;
    if asset.asset_type != AssetType::Image {
        return Ok(());
    }
    let bytes = tokio::fs::read(&asset.original_path).await?;
    let embedding = domus_ml::image_embedding(&bytes, "ViT-B-32__openai")
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    ctx.repos
        .search
        .upsert_smart_embedding(asset.id, &embedding)
        .await
}

/// Permanently remove a trashed asset: delete files, exif, then the row.
async fn asset_deletion(ctx: &JobContext, job: &JobData) -> Result<()> {
    let asset = ctx.repos.asset.get(job_asset_id(job)?).await?;
    let paths = [
        PathBuf::from(&asset.original_path),
        ctx.storage.preview_path(asset.owner_id, asset.id),
        ctx.storage.thumbnail_path(asset.owner_id, asset.id),
        ctx.storage.encoded_video_path(asset.owner_id, asset.id),
    ];
    for path in paths {
        match tokio::fs::remove_file(&path).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(Error::Io(e)),
        }
    }
    ctx.repos.asset.permanently_delete(asset.id).await
}

fn job_asset_id(job: &JobData) -> Result<Uuid> {
    let id = job
        .payload
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::BadRequest("job payload missing asset id".into()))?;
    Uuid::parse_str(id).map_err(|_| Error::BadRequest("invalid asset id".into()))
}

fn parse_exif_datetime(value: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(value, "%Y:%m:%d %H:%M:%S")
                .ok()
                .map(|dt| dt.and_utc())
        })
}

fn duration_millis(seconds: f64) -> i32 {
    (seconds * 1000.0).round().max(0.0).min(i32::MAX as f64) as i32
}
