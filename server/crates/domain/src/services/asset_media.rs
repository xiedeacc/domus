//! Asset binary ingest and serving: upload, original/thumbnail download,
//! video playback and replacement.

use domus_common::types::{AssetType, AssetVisibility};
use domus_common::Result;
use domus_db::repositories::asset::CreateAsset;
use domus_db::Repositories;
use domus_jobs::{JobName, PgJobQueue};
use domus_media::storage::StorageCore;
use std::path::PathBuf;
use uuid::Uuid;

pub struct AssetMediaService {
    repos: Repositories,
    queue: PgJobQueue,
    storage: StorageCore,
}

pub struct UploadRequest {
    pub owner_id: Uuid,
    pub device_asset_id: String,
    pub device_id: String,
    pub file_created_at: chrono::DateTime<chrono::Utc>,
    pub file_modified_at: chrono::DateTime<chrono::Utc>,
    pub filename: String,
    pub is_favorite: bool,
    pub live_photo_video_id: Option<Uuid>,
    /// Temp file the multipart body was streamed to.
    pub staged_file: PathBuf,
    pub checksum: Vec<u8>,
}

pub enum UploadOutcome {
    /// 201 — new asset created.
    Created(Uuid),
    /// 200 — checksum matched an existing asset (client should not re-upload).
    Duplicate(Uuid),
}

impl AssetMediaService {
    pub fn new(repos: Repositories, queue: PgJobQueue, storage: StorageCore) -> Self {
        Self {
            repos,
            queue,
            storage,
        }
    }

    /// Ingest an uploaded file:
    ///   1. dedup by (owner, SHA-1) — return Duplicate without touching disk
    ///   2. move the staged file into upload/<user>/<xx>/<yy>/<uuid>.<ext>
    ///   3. insert the asset row
    ///   4. enqueue MetadataExtraction (which fans out the rest of the
    ///      pipeline: thumbnails → thumbhash → video conversion)
    pub async fn upload(&self, request: UploadRequest) -> Result<UploadOutcome> {
        if let Some(existing) = self
            .repos
            .asset
            .get_by_checksum(request.owner_id, &request.checksum)
            .await?
        {
            return Ok(UploadOutcome::Duplicate(existing));
        }

        let asset_id = Uuid::new_v4();
        let ext = extension(&request.filename);
        let final_path = self
            .storage_path_for_upload(&request, asset_id, &ext)
            .await?;
        if let Some(parent) = final_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        move_file(&request.staged_file, &final_path).await?;

        let asset = self
            .repos
            .asset
            .create(CreateAsset {
                id: asset_id,
                owner_id: request.owner_id,
                device_asset_id: request.device_asset_id,
                device_id: request.device_id,
                asset_type: infer_asset_type(&request.filename),
                original_path: final_path.to_string_lossy().into_owned(),
                original_file_name: request.filename,
                checksum: request.checksum,
                file_created_at: request.file_created_at,
                file_modified_at: request.file_modified_at,
                is_favorite: request.is_favorite,
                live_photo_video_id: request.live_photo_video_id,
                visibility: AssetVisibility::Timeline,
            })
            .await?;

        self.queue
            .enqueue(
                JobName::MetadataExtraction,
                serde_json::json!({ "id": asset.id }),
            )
            .await?;
        Ok(UploadOutcome::Created(asset.id))
    }

    async fn storage_path_for_upload(
        &self,
        request: &UploadRequest,
        asset_id: Uuid,
        ext: &str,
    ) -> Result<PathBuf> {
        let template = self
            .repos
            .system_metadata
            .get("system-config")
            .await?
            .and_then(|config| config.get("storageTemplate").cloned())
            .unwrap_or_else(|| {
                serde_json::json!({
                    "enabled": false,
                    "template": "{{y}}/{{MM}}/{{filename}}",
                })
            });
        if template
            .get("enabled")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            let user = self.repos.user.get(request.owner_id).await?;
            let owner_segment = user
                .storage_label
                .as_deref()
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| request.owner_id.to_string());
            let pattern = template
                .get("template")
                .and_then(|value| value.as_str())
                .filter(|value| !value.is_empty())
                .unwrap_or("{{y}}/{{MM}}/{{filename}}");
            return Ok(self.storage.library_template_path(
                &owner_segment,
                asset_id,
                &request.filename,
                request.file_created_at,
                pattern,
            ));
        }
        Ok(self.storage.upload_path(request.owner_id, asset_id, ext))
    }

    /// Path of the original file for GET /assets/:id/original.
    pub async fn original_path(&self, asset_id: Uuid) -> Result<PathBuf> {
        let asset = self.repos.asset.get(asset_id).await?;
        Ok(PathBuf::from(asset.original_path))
    }

    /// Path of preview/thumbnail for GET /assets/:id/thumbnail?size=...
    pub async fn thumbnail_path(&self, asset_id: Uuid, size: &str) -> Result<PathBuf> {
        let asset = self.repos.asset.get(asset_id).await?;
        let path = match size {
            "preview" | "fullsize" => self.storage.preview_path(asset.owner_id, asset.id),
            _ => self.storage.thumbnail_path(asset.owner_id, asset.id),
        };
        if tokio::fs::metadata(&path).await.is_ok() {
            return Ok(path);
        }
        Ok(PathBuf::from(asset.original_path))
    }

    /// Path of the playback file (encoded video if present, else original)
    /// for GET /assets/:id/video/playback.
    pub async fn playback_path(&self, asset_id: Uuid) -> Result<PathBuf> {
        let asset = self.repos.asset.get(asset_id).await?;
        let encoded = self.storage.encoded_video_path(asset.owner_id, asset.id);
        if tokio::fs::metadata(&encoded).await.is_ok() {
            return Ok(encoded);
        }
        Ok(PathBuf::from(asset.original_path))
    }

    /// POST /assets/exist + POST /assets/bulk-upload-check support.
    pub async fn check_existing(
        &self,
        owner_id: Uuid,
        device_asset_ids: &[String],
    ) -> Result<Vec<String>> {
        self.repos
            .asset
            .find_existing_device_assets(owner_id, device_asset_ids)
            .await
    }
}

fn extension(filename: &str) -> String {
    std::path::Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("bin")
        .to_ascii_lowercase()
}

fn infer_asset_type(filename: &str) -> AssetType {
    match extension(filename).as_str() {
        "3fr" | "ari" | "arw" | "avif" | "bmp" | "cap" | "cin" | "cr2" | "cr3" | "crw" | "dcr"
        | "dng" | "erf" | "fff" | "gif" | "heic" | "heif" | "hif" | "iiq" | "jpe" | "jpeg"
        | "jpg" | "jp2" | "jxl" | "k25" | "kdc" | "mrw" | "nef" | "nrw" | "orf" | "ori" | "pef"
        | "png" | "psd" | "raf" | "raw" | "rwl" | "rw2" | "sr2" | "srf" | "srw" | "svg" | "tif"
        | "tiff" | "webp" | "x3f" => AssetType::Image,
        "3gp" | "3gpp" | "avi" | "flv" | "m2t" | "m2ts" | "m4v" | "mkv" | "mov" | "mp4" | "mpe"
        | "mpeg" | "mpg" | "mts" | "mxf" | "ts" | "vob" | "webm" | "wmv" => AssetType::Video,
        "mp3" | "m4a" | "wav" | "flac" | "aac" | "ogg" => AssetType::Audio,
        _ => AssetType::Other,
    }
}

async fn move_file(from: &std::path::Path, to: &std::path::Path) -> Result<()> {
    match tokio::fs::rename(from, to).await {
        Ok(()) => Ok(()),
        Err(_) => {
            tokio::fs::copy(from, to).await?;
            tokio::fs::remove_file(from).await?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_defaults_and_lowercases() {
        assert_eq!(extension("IMG_0001.JPG"), "jpg");
        assert_eq!(extension("no-extension"), "bin");
        assert_eq!(extension("archive.tar.gz"), "gz");
    }

    #[test]
    fn infer_asset_type_from_common_extensions() {
        assert_eq!(infer_asset_type("photo.heic"), AssetType::Image);
        assert_eq!(infer_asset_type("photo.NEF"), AssetType::Image);
        assert_eq!(infer_asset_type("photo.cr3"), AssetType::Image);
        assert_eq!(infer_asset_type("photo.3fr"), AssetType::Image);
        assert_eq!(infer_asset_type("photo.ari"), AssetType::Image);
        assert_eq!(infer_asset_type("photo.avif"), AssetType::Image);
        assert_eq!(infer_asset_type("photo.jxl"), AssetType::Image);
        assert_eq!(infer_asset_type("photo.x3f"), AssetType::Image);
        assert_eq!(infer_asset_type("clip.MOV"), AssetType::Video);
        assert_eq!(infer_asset_type("clip.M2TS"), AssetType::Video);
        assert_eq!(infer_asset_type("clip.MXF"), AssetType::Video);
        assert_eq!(infer_asset_type("clip.VOB"), AssetType::Video);
        assert_eq!(infer_asset_type("voice.flac"), AssetType::Audio);
        assert_eq!(infer_asset_type("sidecar.xmp"), AssetType::Other);
    }

    #[test]
    fn infer_asset_type_matches_immich_mime_extension_set() {
        for ext in [
            "3fr", "ari", "arw", "avif", "bmp", "cap", "cin", "cr2", "cr3", "crw", "dcr", "dng",
            "erf", "fff", "gif", "heic", "heif", "hif", "iiq", "jp2", "jxl", "k25", "kdc", "mrw",
            "nef", "nrw", "orf", "ori", "pef", "psd", "raf", "raw", "rwl", "sr2", "srf", "srw",
            "svg", "tif", "tiff", "webp", "x3f",
        ] {
            assert_eq!(infer_asset_type(&format!("asset.{ext}")), AssetType::Image);
        }
        for ext in [
            "3gp", "3gpp", "avi", "flv", "m2t", "m2ts", "mkv", "mov", "mp4", "mpe", "mpeg", "mpg",
            "mts", "mxf", "ts", "vob", "webm", "wmv",
        ] {
            assert_eq!(infer_asset_type(&format!("asset.{ext}")), AssetType::Video);
        }
    }
}
