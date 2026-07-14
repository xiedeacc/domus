//! On-disk layout of the media directory. Matches Immich's folder structure
//! so existing installations can be pointed at Domus unchanged:
//!
//! ```text
//! <media>/
//!   upload/<userId>/<xx>/<yy>/<uuid>.<ext>     staging for new uploads
//!   library/<storageLabel|userId>/...          storage-template output
//!   thumbs/<userId>/<xx>/<yy>/<uuid>-{preview.jpeg,thumbnail.webp}
//!   encoded-video/<userId>/<xx>/<yy>/<uuid>.mp4
//!   profile/<userId>/<uuid>.<ext>
//!   backups/                                    database dumps
//! ```

use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum StorageFolder {
    Upload,
    Library,
    Thumbs,
    EncodedVideo,
    Profile,
    Backups,
}

impl StorageFolder {
    pub fn as_str(&self) -> &'static str {
        match self {
            StorageFolder::Upload => "upload",
            StorageFolder::Library => "library",
            StorageFolder::Thumbs => "thumbs",
            StorageFolder::EncodedVideo => "encoded-video",
            StorageFolder::Profile => "profile",
            StorageFolder::Backups => "backups",
        }
    }
}

#[derive(Clone)]
pub struct StorageCore {
    media_root: PathBuf,
}

impl StorageCore {
    pub fn new(media_root: impl Into<PathBuf>) -> Self {
        Self { media_root: media_root.into() }
    }

    pub fn folder(&self, folder: StorageFolder) -> PathBuf {
        self.media_root.join(folder.as_str())
    }

    /// Two-level fan-out dir derived from the asset UUID (e.g. `ab/cd`),
    /// keeping directory sizes bounded — same scheme Immich uses.
    fn fanout(id: Uuid) -> (String, String) {
        let s = id.to_string();
        (s[0..2].to_string(), s[2..4].to_string())
    }

    pub fn upload_path(&self, user_id: Uuid, asset_id: Uuid, ext: &str) -> PathBuf {
        let (a, b) = Self::fanout(asset_id);
        self.folder(StorageFolder::Upload)
            .join(user_id.to_string())
            .join(a)
            .join(b)
            .join(format!("{asset_id}.{ext}"))
    }

    pub fn preview_path(&self, user_id: Uuid, asset_id: Uuid) -> PathBuf {
        let (a, b) = Self::fanout(asset_id);
        self.folder(StorageFolder::Thumbs)
            .join(user_id.to_string())
            .join(a)
            .join(b)
            .join(format!("{asset_id}-preview.jpeg"))
    }

    pub fn thumbnail_path(&self, user_id: Uuid, asset_id: Uuid) -> PathBuf {
        let (a, b) = Self::fanout(asset_id);
        self.folder(StorageFolder::Thumbs)
            .join(user_id.to_string())
            .join(a)
            .join(b)
            .join(format!("{asset_id}-thumbnail.webp"))
    }

    pub fn encoded_video_path(&self, user_id: Uuid, asset_id: Uuid) -> PathBuf {
        let (a, b) = Self::fanout(asset_id);
        self.folder(StorageFolder::EncodedVideo)
            .join(user_id.to_string())
            .join(a)
            .join(b)
            .join(format!("{asset_id}.mp4"))
    }

    pub fn media_root(&self) -> &Path {
        &self.media_root
    }
}
