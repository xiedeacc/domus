//! Video probing and transcoding via ffprobe/ffmpeg, mirroring Immich's
//! policy-driven transcode settings (target codec, resolution, CRF, hwaccel).

use domus_common::{Error, Result};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct VideoInfo {
    pub duration_seconds: f64,
    pub width: i32,
    pub height: i32,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub fps: Option<f64>,
    pub rotation: i32,
}

/// `ffprobe -show_streams -show_format -print_format json`
pub async fn probe(_path: &Path) -> Result<VideoInfo> {
    Err(Error::NotImplemented("transcode::probe"))
}

#[derive(Debug, Clone)]
pub struct TranscodeOptions {
    pub target_video_codec: String, // h264 | hevc | vp9 | av1
    pub target_audio_codec: String, // aac | mp3 | opus
    pub target_resolution: String,  // "720" | "1080" | "original" ...
    pub crf: u8,
    pub two_pass: bool,
}

impl Default for TranscodeOptions {
    fn default() -> Self {
        Self {
            target_video_codec: "h264".into(),
            target_audio_codec: "aac".into(),
            target_resolution: "720".into(),
            crf: 23,
            two_pass: false,
        }
    }
}

/// Transcode `input` into an MP4 at `output` (encoded-video path).
pub async fn transcode(_input: &Path, _output: &Path, _options: TranscodeOptions) -> Result<()> {
    Err(Error::NotImplemented("transcode::transcode"))
}
