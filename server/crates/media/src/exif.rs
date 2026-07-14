//! Metadata extraction. Immich uses exiftool; we shell out to the same tool
//! so tag coverage (RAW formats, sidecars, motion photos) stays identical.

use domus_common::{Error, Result};
use serde::Deserialize;
use std::path::Path;

/// Subset of exiftool output Domus consumes (mirrors ImmichTags).
#[derive(Debug, Default, Deserialize)]
pub struct ExifData {
    #[serde(rename = "Make")]
    pub make: Option<String>,
    #[serde(rename = "Model")]
    pub model: Option<String>,
    #[serde(rename = "ImageWidth")]
    pub image_width: Option<i32>,
    #[serde(rename = "ImageHeight")]
    pub image_height: Option<i32>,
    #[serde(rename = "DateTimeOriginal")]
    pub date_time_original: Option<String>,
    #[serde(rename = "OffsetTimeOriginal")]
    pub offset_time_original: Option<String>,
    #[serde(rename = "GPSLatitude")]
    pub gps_latitude: Option<f64>,
    #[serde(rename = "GPSLongitude")]
    pub gps_longitude: Option<f64>,
    #[serde(rename = "Orientation")]
    pub orientation: Option<i32>,
    #[serde(rename = "ISO")]
    pub iso: Option<i32>,
    #[serde(rename = "FNumber")]
    pub f_number: Option<f64>,
    #[serde(rename = "FocalLength")]
    pub focal_length: Option<f64>,
    #[serde(rename = "ExposureTime")]
    pub exposure_time: Option<String>,
    #[serde(rename = "LensModel")]
    pub lens_model: Option<String>,
    #[serde(rename = "Rating")]
    pub rating: Option<i32>,
}

/// Run `exiftool -json` on the file and parse the result.
pub async fn extract(_path: &Path) -> Result<ExifData> {
    // TODO: spawn `exiftool -json -n <path>` and deserialize.
    Err(Error::NotImplemented("exif::extract"))
}
