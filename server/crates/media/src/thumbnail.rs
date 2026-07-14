//! Thumbnail/preview generation. Immich generates, per asset:
//!   - preview:   JPEG, max 1440px
//!   - thumbnail: WEBP, max 250px
//!   - thumbhash: compact placeholder hash stored on the asset row
//! Video assets get their poster frame extracted with ffmpeg first.

use domus_common::{Error, Result};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Jpeg,
    Webp,
}

#[derive(Debug, Clone)]
pub struct ThumbnailOptions {
    pub size: u32,
    pub format: ImageFormat,
    pub quality: u8,
}

impl ThumbnailOptions {
    pub fn preview() -> Self {
        Self { size: 1440, format: ImageFormat::Jpeg, quality: 80 }
    }
    pub fn thumbnail() -> Self {
        Self { size: 250, format: ImageFormat::Webp, quality: 80 }
    }
}

/// Generate a resized image at `output`, honouring EXIF orientation.
pub async fn generate(_input: &Path, _output: &Path, _options: ThumbnailOptions) -> Result<()> {
    // TODO: use libvips bindings or `image`/`zune` crates; fall back to
    // `vipsthumbnail` CLI for exotic formats.
    Err(Error::NotImplemented("thumbnail::generate"))
}

/// Compute the thumbhash placeholder from the generated thumbnail.
pub async fn thumbhash(_thumbnail: &Path) -> Result<Vec<u8>> {
    Err(Error::NotImplemented("thumbnail::thumbhash"))
}
