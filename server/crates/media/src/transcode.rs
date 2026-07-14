//! Video probing and transcoding via ffprobe/ffmpeg, mirroring Immich's
//! policy-driven transcode settings (target codec, resolution, CRF, hwaccel).

use domus_common::{Error, Result};
use std::path::Path;
use tokio::process::Command;

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
pub async fn probe(path: &Path) -> Result<VideoInfo> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_streams")
        .arg("-show_format")
        .arg("-print_format")
        .arg("json")
        .arg(path)
        .output()
        .await
        .map_err(|e| Error::Internal(e.into()))?;
    if !output.status.success() {
        return Err(Error::Internal(anyhow::anyhow!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| Error::Internal(e.into()))?;
    Ok(parse_probe_json(&value))
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
pub async fn transcode(input: &Path, output: &Path, options: TranscodeOptions) -> Result<()> {
    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut command = Command::new("ffmpeg");
    command
        .arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-movflags")
        .arg("+faststart")
        .arg("-c:v")
        .arg(match options.target_video_codec.as_str() {
            "hevc" => "libx265",
            "vp9" => "libvpx-vp9",
            "av1" => "libaom-av1",
            _ => "libx264",
        })
        .arg("-preset")
        .arg("veryfast")
        .arg("-crf")
        .arg(options.crf.to_string())
        .arg("-c:a")
        .arg(match options.target_audio_codec.as_str() {
            "mp3" => "libmp3lame",
            "opus" => "libopus",
            _ => "aac",
        });

    if options.target_resolution != "original" {
        if let Ok(height) = options.target_resolution.parse::<u32>() {
            command
                .arg("-vf")
                .arg(format!("scale=-2:'min({height},ih)'"));
        }
    }

    let status = command
        .arg(output)
        .status()
        .await
        .map_err(|e| Error::Internal(e.into()))?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Internal(anyhow::anyhow!("ffmpeg transcode failed")))
    }
}

fn parse_probe_json(value: &serde_json::Value) -> VideoInfo {
    let streams = value["streams"].as_array().cloned().unwrap_or_default();
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"].as_str() == Some("video"));
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"].as_str() == Some("audio"));
    let duration_seconds = value["format"]["duration"]
        .as_str()
        .and_then(|v| v.parse().ok())
        .or_else(|| video.and_then(|v| v["duration"].as_str()?.parse().ok()))
        .unwrap_or_default();
    let fps = video.and_then(|v| parse_ratio(v["avg_frame_rate"].as_str()?));
    VideoInfo {
        duration_seconds,
        width: video.and_then(|v| v["width"].as_i64()).unwrap_or_default() as i32,
        height: video.and_then(|v| v["height"].as_i64()).unwrap_or_default() as i32,
        video_codec: video.and_then(|v| v["codec_name"].as_str().map(str::to_owned)),
        audio_codec: audio.and_then(|v| v["codec_name"].as_str().map(str::to_owned)),
        fps,
        rotation: video
            .and_then(|v| v["tags"]["rotate"].as_str()?.parse().ok())
            .unwrap_or_default(),
    }
}

fn parse_ratio(value: &str) -> Option<f64> {
    let (num, den) = value.split_once('/')?;
    let num: f64 = num.parse().ok()?;
    let den: f64 = den.parse().ok()?;
    (den != 0.0).then_some(num / den)
}
