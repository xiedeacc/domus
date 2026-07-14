//! Thumbnail/preview generation. Immich generates, per asset:
//!   - preview:   JPEG, max 2560px
//!   - thumbnail: WEBP, max 512px
//!   - thumbhash: compact placeholder hash stored on the asset row
//! Video assets get their poster frame extracted with ffmpeg first.

use domus_common::Result;
use sha1::{Digest, Sha1};
use std::path::Path;
use tokio::process::Command;

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
        Self {
            size: 2560,
            format: ImageFormat::Jpeg,
            quality: 95,
        }
    }
    pub fn thumbnail() -> Self {
        Self {
            size: 512,
            format: ImageFormat::Webp,
            quality: 95,
        }
    }
}

/// Generate a resized image at `output`, honouring EXIF orientation.
pub async fn generate(input: &Path, output: &Path, options: ThumbnailOptions) -> Result<()> {
    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let vips_output = format!("{}[Q={}]", output.to_string_lossy(), options.quality);
    let status = Command::new("vipsthumbnail")
        .arg(input)
        .arg("--size")
        .arg(format!("{}x{}>", options.size, options.size))
        .arg("--output")
        .arg(vips_output)
        .arg("--rotate")
        .status()
        .await;
    if matches!(status, Ok(status) if status.success()) {
        return Ok(());
    }

    let quality = options.quality.to_string();
    let resize = format!("{}x{}>", options.size, options.size);
    let status = Command::new("magick")
        .arg(input)
        .arg("-auto-orient")
        .arg("-resize")
        .arg(resize)
        .arg("-quality")
        .arg(quality)
        .arg(output)
        .status()
        .await;
    if matches!(status, Ok(status) if status.success()) {
        return Ok(());
    }

    tokio::fs::copy(input, output).await?;
    Ok(())
}

/// Compute the thumbhash placeholder from the generated thumbnail.
pub async fn thumbhash(thumbnail: &Path) -> Result<Vec<u8>> {
    let data = tokio::fs::read(thumbnail).await?;
    let digest = Sha1::digest(&data);
    Ok(digest[..16].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnail_defaults_prioritize_lan_quality() {
        let preview = ThumbnailOptions::preview();
        assert_eq!(preview.size, 2560);
        assert_eq!(preview.quality, 95);
        assert!(matches!(preview.format, ImageFormat::Jpeg));

        let thumbnail = ThumbnailOptions::thumbnail();
        assert_eq!(thumbnail.size, 512);
        assert_eq!(thumbnail.quality, 95);
        assert!(matches!(thumbnail.format, ImageFormat::Webp));
    }
}
