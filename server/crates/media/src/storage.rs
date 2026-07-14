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
        Self {
            media_root: media_root.into(),
        }
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

    pub fn library_template_path(
        &self,
        owner_segment: &str,
        asset_id: Uuid,
        original_filename: &str,
        taken_at: chrono::DateTime<chrono::Utc>,
        template: &str,
    ) -> PathBuf {
        use chrono::Datelike;
        let filename = sanitize_segment(original_filename);
        let stem = Path::new(original_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(sanitize_segment)
            .unwrap_or_else(|| asset_id.to_string());
        let ext = Path::new(original_filename)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_else(|| "bin".to_owned());
        let rendered = template
            .replace("{{y}}", &format!("{:04}", taken_at.year()))
            .replace("{{yyyy}}", &format!("{:04}", taken_at.year()))
            .replace("{{MM}}", &format!("{:02}", taken_at.month()))
            .replace("{{dd}}", &format!("{:02}", taken_at.day()))
            .replace("{{filename}}", &filename)
            .replace("{{fileName}}", &filename)
            .replace("{{name}}", &stem)
            .replace("{{ext}}", &ext)
            .replace("{{assetId}}", &asset_id.to_string());
        self.folder(StorageFolder::Library)
            .join(sanitize_segment(owner_segment))
            .join(rendered)
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

    pub fn is_immich_path(&self, path: impl AsRef<Path>) -> bool {
        let media_root = normalize_for_prefix(&self.media_root);
        let path = normalize_for_prefix(path.as_ref());
        path == media_root || path.starts_with(&format!("{media_root}/"))
    }
}

fn sanitize_segment(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

fn normalize_for_prefix(path: &Path) -> String {
    path.to_string_lossy().trim_end_matches('/').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_immich_path_matches_media_location() {
        let storage = StorageCore::new("/photos");

        assert!(storage.is_immich_path("/photos"));
        assert!(storage.is_immich_path("/photos/new/"));
        assert!(!storage.is_immich_path("/photos_new"));
        assert!(!storage.is_immich_path("/some/other/path"));
    }

    #[test]
    fn library_template_path_sanitizes_segments_and_renders_immich_tokens() {
        let storage = StorageCore::new("/photos");
        let asset_id = Uuid::parse_str("10000000-0000-0000-0000-000000000000").unwrap();
        let taken_at = chrono::DateTime::parse_from_rfc3339("2026-07-14T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        let path = storage.library_template_path(
            "label/with:bad",
            asset_id,
            "IMG:0001.JPG",
            taken_at,
            "{{y}}/{{MM}}/{{dd}}/{{name}}-{{assetId}}.{{ext}}",
        );

        assert_eq!(
            path.to_string_lossy(),
            "/photos/library/label_with_bad/2026/07/14/IMG_0001-10000000-0000-0000-0000-000000000000.jpg"
        );
    }
}
