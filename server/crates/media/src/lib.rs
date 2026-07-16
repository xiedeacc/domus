//! Media processing: EXIF extraction, thumbnail generation and video
//! transcoding. All heavy lifting is delegated to external tools (exiftool,
//! ffmpeg/ffprobe) or native libraries behind these facades, matching the
//! pipeline Immich runs (exiftool-vendored + sharp/libvips + ffmpeg).

pub mod exif;
pub mod immich_derivatives;
pub mod storage;
pub mod thumbnail;
pub mod transcode;

use domus_common::Result;
use std::path::Path;

/// SHA-1 checksum of a file — Immich's asset identity for dedup and the
/// `x-immich-checksum` upload header.
pub async fn sha1_checksum(path: &Path) -> Result<Vec<u8>> {
    use sha1::{Digest, Sha1};
    let data = tokio::fs::read(path).await?;
    let mut hasher = Sha1::new();
    hasher.update(&data);
    Ok(hasher.finalize().to_vec())
}
