use domus_common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tempfile::TempDir;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const DEFAULT_STATE_DIR: &str = "/var/lib/immich_derivative_repair";
const DEFAULT_UPLOAD_ROOT: &str = "/opt/immich/upload";
const DEFAULT_LIMIT: usize = 200;
const MIN_THUMBNAIL_BYTES: u64 = 1000;
const MIN_PREVIEW_BYTES: u64 = 50_000;
const KNOWN_PROBLEM_PATHS: &[&str] = &[
    "/opt/immich/upload/library/admin/2026/2026-07-12/6390.heic",
    "/opt/immich/upload/library/admin/2026/2026-07-11/IMG_1229.mov",
    "/opt/immich/upload/library/admin/2026/2026-07-11/IMG_1228.mov",
    "/opt/immich/upload/library/admin/2026/2026-06-07/IMG_1101.heic",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichDerivativeRequest {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub repair_all: bool,
    #[serde(default)]
    pub shard_index: usize,
    #[serde(default = "default_shard_count")]
    pub shard_count: usize,
    #[serde(default = "default_resume")]
    pub resume: bool,
    #[serde(default)]
    pub asset_ids: Vec<String>,
    #[serde(default)]
    pub upload_root: Option<String>,
    #[serde(default)]
    pub state_dir: Option<String>,
}

impl Default for ImmichDerivativeRequest {
    fn default() -> Self {
        Self {
            limit: Some(DEFAULT_LIMIT),
            repair_all: false,
            shard_index: 0,
            shard_count: 1,
            resume: true,
            asset_ids: Vec::new(),
            upload_root: None,
            state_dir: None,
        }
    }
}

fn default_shard_count() -> usize {
    1
}
fn default_resume() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichDerivativeSummary {
    pub ok: usize,
    pub failed: usize,
    pub resumed: usize,
    pub checked: usize,
    pub shard_index: usize,
    pub shard_count: usize,
    pub cancelled: bool,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichDerivativeProgress {
    pub checked: usize,
    pub total: usize,
    pub ok: usize,
    pub failed: usize,
    pub resumed: usize,
    pub current_asset_id: Option<String>,
    pub phase: String,
    pub recent_messages: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RepairProgress {
    inner: Arc<tokio::sync::Mutex<ImmichDerivativeProgress>>,
}

impl RepairProgress {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(ImmichDerivativeProgress {
                phase: "idle".to_owned(),
                ..Default::default()
            })),
        }
    }

    pub async fn snapshot(&self) -> ImmichDerivativeProgress {
        self.inner.lock().await.clone()
    }

    async fn set_total(&self, total: usize) {
        self.inner.lock().await.total = total;
    }

    async fn phase(&self, phase: impl Into<String>, asset_id: Option<String>) {
        let mut guard = self.inner.lock().await;
        guard.phase = phase.into();
        guard.current_asset_id = asset_id;
    }

    async fn counts(&self, ok: usize, failed: usize, resumed: usize, checked: usize) {
        let mut guard = self.inner.lock().await;
        guard.ok = ok;
        guard.failed = failed;
        guard.resumed = resumed;
        guard.checked = checked;
    }

    async fn message(&self, message: impl Into<String>) {
        let mut guard = self.inner.lock().await;
        guard.recent_messages.push(message.into());
        if guard.recent_messages.len() > 40 {
            guard.recent_messages.remove(0);
        }
    }
}

impl Default for ImmichDerivativeProgress {
    fn default() -> Self {
        Self {
            checked: 0,
            total: 0,
            ok: 0,
            failed: 0,
            resumed: 0,
            current_asset_id: None,
            phase: "idle".to_owned(),
            recent_messages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct AssetRow {
    id: String,
    owner_id: String,
    asset_type: String,
    original_path: PathBuf,
    thumb_path: Option<PathBuf>,
    preview_path: Option<PathBuf>,
    fullsize_path: Option<PathBuf>,
    thumb_bytes: Option<u64>,
    preview_bytes: Option<u64>,
    fullsize_bytes: Option<u64>,
    ext: String,
    duration: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct Fingerprint {
    path: String,
    size: u64,
    mtime_ns: i128,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct StrategyEntry {
    fingerprint: Fingerprint,
    status: String,
    strategy: Option<String>,
}

type StrategyCache = HashMap<String, StrategyEntry>;

pub async fn run_repair(
    request: ImmichDerivativeRequest,
    progress: RepairProgress,
    cancel: Arc<AtomicBool>,
) -> Result<ImmichDerivativeSummary> {
    let upload_root = PathBuf::from(
        request
            .upload_root
            .as_deref()
            .unwrap_or(DEFAULT_UPLOAD_ROOT),
    );
    let state_dir = PathBuf::from(request.state_dir.as_deref().unwrap_or(DEFAULT_STATE_DIR));
    fs::create_dir_all(&state_dir).await?;
    let shard_count = request.shard_count.max(1);
    let shard_index = request.shard_index.min(shard_count - 1);
    let mut summary = ImmichDerivativeSummary {
        shard_index,
        shard_count,
        ..Default::default()
    };

    progress.phase("querying Immich database", None).await;
    let mut rows = query_assets(&request, shard_index, shard_count).await?;
    if !request.repair_all && request.asset_ids.is_empty() {
        rows.truncate(request.limit.unwrap_or(DEFAULT_LIMIT));
    }
    summary.checked = rows.len();
    progress.set_total(rows.len()).await;
    progress
        .message(format!("loaded {} candidate asset(s)", rows.len()))
        .await;

    let checkpoint_path = state_dir.join(format!(
        "repair_all_shard_{}_of_{}.ok",
        shard_index, shard_count
    ));
    let mut completed = if request.resume && request.repair_all {
        read_checkpoint(&checkpoint_path).await?
    } else {
        HashSet::new()
    };
    let cache_path = state_dir.join("strategy-cache.json");
    let mut strategy_cache = read_strategy_cache(&cache_path).await?;

    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut resumed = 0usize;
    for row in rows.iter() {
        if cancel.load(Ordering::Relaxed) {
            summary.cancelled = true;
            progress
                .message("cancel requested; stopping after current asset list checkpoint")
                .await;
            break;
        }
        progress
            .phase("checking required derivatives", Some(row.id.clone()))
            .await;
        if completed.contains(&row.id) {
            resumed += 1;
            progress
                .counts(ok, failed, resumed, ok + failed + resumed)
                .await;
            continue;
        }
        match repair_asset(row, &upload_root, &mut strategy_cache, &progress).await {
            Ok(true) => {
                ok += 1;
                append_checkpoint(&checkpoint_path, &row.id, "ok").await?;
                completed.insert(row.id.clone());
            }
            Ok(false) => {
                ok += 1;
                append_checkpoint(&checkpoint_path, &row.id, "skipped").await?;
                completed.insert(row.id.clone());
            }
            Err(err) => {
                failed += 1;
                let msg = format!("{} failed: {}", row.id, err);
                progress.message(&msg).await;
                summary.messages.push(msg);
            }
        }
        write_strategy_cache(&cache_path, &strategy_cache).await?;
        progress
            .counts(ok, failed, resumed, ok + failed + resumed)
            .await;
    }

    if request.resume && request.repair_all && failed == 0 && !summary.cancelled {
        let _ = fs::remove_file(&checkpoint_path).await;
    }
    summary.ok = ok;
    summary.failed = failed;
    summary.resumed = resumed;
    summary.checked = rows.len();
    progress.phase("done", None).await;
    progress
        .message(format!(
        "immich derivative repair summary: shard {}/{}, {} ok, {} failed, {} resumed, {} checked",
        shard_index, shard_count, ok, failed, resumed, rows.len()
    ))
        .await;
    Ok(summary)
}

async fn query_assets(
    request: &ImmichDerivativeRequest,
    shard_index: usize,
    shard_count: usize,
) -> Result<Vec<AssetRow>> {
    let mut where_parts = Vec::new();
    if !request.asset_ids.is_empty() {
        let ids = request
            .asset_ids
            .iter()
            .map(|id| sql_quote(id))
            .collect::<Vec<_>>()
            .join(",");
        where_parts.push(format!("a.id::text IN ({ids})"));
    } else if request.repair_all {
        where_parts.push("a.type IN ('IMAGE','VIDEO')".to_owned());
    } else {
        let known = KNOWN_PROBLEM_PATHS
            .iter()
            .map(|p| sql_quote(p))
            .collect::<Vec<_>>()
            .join(",");
        where_parts.push(format!("((a.type = 'VIDEO' AND (thumb.path IS NULL OR preview.path IS NULL)) OR (lower(a.originalPath) ~ '\\.(heic|heif)$' AND (thumb.path IS NULL OR preview.path IS NULL OR fullsize.path IS NULL OR COALESCE(thumb.bytes,0) < {MIN_THUMBNAIL_BYTES} OR COALESCE(preview.bytes,0) < {MIN_PREVIEW_BYTES} OR COALESCE(fullsize.bytes,0) < {MIN_PREVIEW_BYTES})) OR a.originalPath IN ({known}))"));
    }
    if shard_count > 1 {
        where_parts.push(format!(
            "mod(abs(hashtext(a.id::text)), {shard_count}) = {shard_index}"
        ));
    }
    let limit = if request.repair_all || !request.asset_ids.is_empty() {
        String::new()
    } else {
        format!(" LIMIT {}", request.limit.unwrap_or(DEFAULT_LIMIT))
    };
    let sql = format!(
        r#"
SELECT a.id::text, a."ownerId"::text, a.type::text, a."originalPath"::text,
       thumb.path::text, preview.path::text, fullsize.path::text,
       thumb.bytes::text, preview.bytes::text, fullsize.bytes::text,
       lower(coalesce(a."originalFileName", a."originalPath"))::text,
       coalesce(a.duration::text, '')
FROM asset a
LEFT JOIN asset_file thumb ON thumb."assetId" = a.id AND thumb.type = 'THUMBNAIL' AND thumb."isEdited" = false
LEFT JOIN asset_file preview ON preview."assetId" = a.id AND preview.type = 'PREVIEW' AND preview."isEdited" = false
LEFT JOIN asset_file fullsize ON fullsize."assetId" = a.id AND fullsize.type = 'FULLSIZE' AND fullsize."isEdited" = false
WHERE a."deletedAt" IS NULL AND ({})
ORDER BY a."createdAt" ASC{}
"#,
        where_parts.join(" AND "),
        limit
    );
    let output = psql_query(&sql).await?;
    let mut rows = Vec::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 12 {
            continue;
        }
        rows.push(AssetRow {
            id: cols[0].to_owned(),
            owner_id: cols[1].to_owned(),
            asset_type: cols[2].to_owned(),
            original_path: PathBuf::from(cols[3]),
            thumb_path: non_empty(cols[4]).map(PathBuf::from),
            preview_path: non_empty(cols[5]).map(PathBuf::from),
            fullsize_path: non_empty(cols[6]).map(PathBuf::from),
            thumb_bytes: parse_u64(cols[7]),
            preview_bytes: parse_u64(cols[8]),
            fullsize_bytes: parse_u64(cols[9]),
            ext: Path::new(cols[10])
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("")
                .to_ascii_lowercase(),
            duration: parse_duration(cols[11]),
        });
    }
    Ok(rows)
}

fn sql_quote(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "''"))
}

fn non_empty(value: &str) -> Option<&str> {
    (!value.is_empty() && value != "\\N").then_some(value)
}
fn parse_u64(value: &str) -> Option<u64> {
    non_empty(value)?.parse().ok()
}
fn parse_duration(value: &str) -> Option<f64> {
    non_empty(value)?.parse().ok()
}

async fn repair_asset(
    row: &AssetRow,
    upload_root: &Path,
    cache: &mut StrategyCache,
    progress: &RepairProgress,
) -> Result<bool> {
    if fs::metadata(&row.original_path)
        .await
        .map(|m| m.len())
        .unwrap_or(0)
        == 0
    {
        progress
            .message(format!("{} original is empty or missing", row.id))
            .await;
        return Ok(false);
    }
    let thumb_out = derivative_path(upload_root, row, "thumbnail.webp");
    let preview_out = derivative_path(upload_root, row, "preview.jpeg");
    let fullsize_out = derivative_path(upload_root, row, "fullsize.jpeg");

    let need_thumb = needs_derivative(
        row.thumb_path.as_ref(),
        row.thumb_bytes,
        MIN_THUMBNAIL_BYTES,
        &thumb_out,
    )
    .await;
    let need_preview = needs_derivative(
        row.preview_path.as_ref(),
        row.preview_bytes,
        MIN_PREVIEW_BYTES,
        &preview_out,
    )
    .await;
    let need_fullsize = row.asset_type == "IMAGE"
        && needs_derivative(
            row.fullsize_path.as_ref(),
            row.fullsize_bytes,
            MIN_PREVIEW_BYTES,
            &fullsize_out,
        )
        .await;
    if !need_thumb && !need_preview && !need_fullsize {
        return Ok(false);
    }

    let tmp = TempDir::new()?;
    let image_source = if row.asset_type == "VIDEO" {
        progress
            .phase(
                "extracting representative video frame",
                Some(row.id.clone()),
            )
            .await;
        representative_video_frame(row, tmp.path()).await?
    } else if is_heic(row) {
        progress
            .phase(
                "decoding HEIC/HEIF with fast and fallback strategies",
                Some(row.id.clone()),
            )
            .await;
        decode_heic(row, tmp.path(), cache).await?
    } else {
        row.original_path.clone()
    };

    let mut changed = false;
    if need_thumb {
        progress
            .phase("writing thumbnail webp", Some(row.id.clone()))
            .await;
        make_image(&image_source, &thumb_out, 512, true).await?;
        upsert_asset_file(&row.id, "THUMBNAIL", &thumb_out).await?;
        changed = true;
    }
    if need_preview {
        progress
            .phase("writing preview jpeg", Some(row.id.clone()))
            .await;
        make_image(&image_source, &preview_out, 2048, false).await?;
        upsert_asset_file(&row.id, "PREVIEW", &preview_out).await?;
        changed = true;
    }
    if need_fullsize {
        progress
            .phase("writing fullsize jpeg", Some(row.id.clone()))
            .await;
        make_image(&image_source, &fullsize_out, 4096, false).await?;
        upsert_asset_file(&row.id, "FULLSIZE", &fullsize_out).await?;
        changed = true;
    }
    if changed {
        refresh_asset(
            &row.id,
            if need_thumb {
                Some(&thumb_out)
            } else {
                row.thumb_path.as_deref()
            },
        )
        .await?;
    }
    Ok(changed)
}

fn derivative_path(upload_root: &Path, row: &AssetRow, suffix: &str) -> PathBuf {
    let id = &row.id;
    let a = id.get(0..2).unwrap_or("00");
    let b = id.get(2..4).unwrap_or("00");
    upload_root
        .join("thumbs")
        .join(&row.owner_id)
        .join(a)
        .join(b)
        .join(format!("{id}_{suffix}"))
}

async fn needs_derivative(
    db_path: Option<&PathBuf>,
    db_bytes: Option<u64>,
    min_bytes: u64,
    target: &Path,
) -> bool {
    let path = db_path.unwrap_or(&target.to_path_buf()).clone();
    if db_bytes.unwrap_or(0) < min_bytes {
        return true;
    }
    fs::metadata(path)
        .await
        .map(|m| m.len() < min_bytes)
        .unwrap_or(true)
}

fn is_heic(row: &AssetRow) -> bool {
    matches!(row.ext.as_str(), "heic" | "heif")
}

async fn make_image(input: &Path, output: &Path, size: u32, thumbnail: bool) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).await?;
    }
    let resize = format!("{}x{}>", size, size);
    let quality = if thumbnail { "95" } else { "92" };
    if command_ok(
        Command::new("magick")
            .arg(input)
            .arg("-auto-orient")
            .arg("-resize")
            .arg(&resize)
            .arg("-quality")
            .arg(quality)
            .arg(output),
    )
    .await
    {
        finish_file(output).await?;
        return Ok(());
    }
    if command_ok(
        Command::new("convert")
            .arg(input)
            .arg("-auto-orient")
            .arg("-resize")
            .arg(&resize)
            .arg("-quality")
            .arg(quality)
            .arg(output),
    )
    .await
    {
        finish_file(output).await?;
        return Ok(());
    }
    let scale = format!("scale='if(gt(iw,ih),-2,{size})':'if(gt(iw,ih),{size},-2)'");
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .arg("-frames:v")
        .arg("1")
        .arg("-vf")
        .arg(scale);
    if thumbnail {
        cmd.arg("-quality").arg("95");
    } else {
        cmd.arg("-q:v").arg("1");
    }
    cmd.arg(output);
    if command_ok(&mut cmd).await {
        finish_file(output).await?;
        return Ok(());
    }
    Err(Error::Internal(anyhow::anyhow!(
        "failed to generate {}",
        output.display()
    )))
}

async fn decode_heic(row: &AssetRow, tmp: &Path, cache: &mut StrategyCache) -> Result<PathBuf> {
    let fp = fingerprint(&row.original_path).await?;
    if let Some(entry) = cache.get(&row.id) {
        if entry.fingerprint == fp && entry.status == "ok" {
            if let Some(strategy) = &entry.strategy {
                if let Ok(path) = run_heic_strategy(row, tmp, strategy).await {
                    return Ok(path);
                }
            }
        }
    }
    for strategy in ["box-grid", "normal", "primary-stream", "tile-grid"] {
        if let Ok(path) = run_heic_strategy(row, tmp, strategy).await {
            cache.insert(
                row.id.clone(),
                StrategyEntry {
                    fingerprint: fp,
                    status: "ok".to_owned(),
                    strategy: Some(strategy.to_owned()),
                },
            );
            return Ok(path);
        }
    }
    cache.insert(
        row.id.clone(),
        StrategyEntry {
            fingerprint: fp,
            status: "known-failed".to_owned(),
            strategy: None,
        },
    );
    Err(Error::Internal(anyhow::anyhow!(
        "all HEIC decode strategies failed for {}",
        row.original_path.display()
    )))
}

async fn run_heic_strategy(row: &AssetRow, tmp: &Path, strategy: &str) -> Result<PathBuf> {
    match strategy {
        "box-grid" => decode_heic_box_grid(row, tmp).await,
        "normal" => decode_heic_normal(row, tmp).await,
        "primary-stream" => decode_heic_primary_stream(row, tmp).await,
        "tile-grid" => decode_heic_tile_grid(row, tmp).await,
        _ => Err(Error::BadRequest(format!(
            "unknown HEIC strategy {strategy}"
        ))),
    }
}

async fn decode_heic_box_grid(row: &AssetRow, tmp: &Path) -> Result<PathBuf> {
    // The old Python implementation used a hand parser for HEIF grid boxes.
    // In Rust we first ask ImageMagick/libheif for the primary grid decode;
    // if the file is one of the broken grid HEICs this fails quickly and the
    // later primary-stream/tile-grid strategies take over.
    let out = tmp.join("box_grid_full.jpg");
    if command_ok(
        Command::new("magick")
            .arg(format!("{}[0]", row.original_path.display()))
            .arg("-auto-orient")
            .arg(&out),
    )
    .await
    {
        return Ok(out);
    }
    Err(Error::Internal(anyhow::anyhow!("box-grid decode failed")))
}

async fn decode_heic_normal(row: &AssetRow, tmp: &Path) -> Result<PathBuf> {
    let out = tmp.join("normal.jpg");
    if command_ok(
        Command::new("magick")
            .arg(&row.original_path)
            .arg("-auto-orient")
            .arg(&out),
    )
    .await
    {
        return Ok(out);
    }
    if command_ok(
        Command::new("convert")
            .arg(&row.original_path)
            .arg("-auto-orient")
            .arg(&out),
    )
    .await
    {
        return Ok(out);
    }
    if command_ok(
        Command::new("heif-convert")
            .arg(&row.original_path)
            .arg(&out),
    )
    .await
    {
        return Ok(out);
    }
    Err(Error::Internal(anyhow::anyhow!(
        "normal HEIC decode failed"
    )))
}

async fn decode_heic_primary_stream(row: &AssetRow, tmp: &Path) -> Result<PathBuf> {
    let out = tmp.join("primary_stream.jpg");
    let stream = first_video_stream(&row.original_path).await?.unwrap_or(0);
    let map = format!("0:{stream}");
    if command_ok(
        Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-y")
            .arg("-i")
            .arg(&row.original_path)
            .arg("-map")
            .arg(map)
            .arg("-frames:v")
            .arg("1")
            .arg("-q:v")
            .arg("1")
            .arg(&out),
    )
    .await
    {
        return Ok(out);
    }
    Err(Error::Internal(anyhow::anyhow!(
        "primary stream decode failed"
    )))
}

async fn decode_heic_tile_grid(row: &AssetRow, tmp: &Path) -> Result<PathBuf> {
    // Preserve the old fallback shape while keeping it bounded: extract the
    // largest video stream frame. Broken tiled HEICs that expose only tiles are
    // still handled by ffmpeg's decoder when possible, without probing forever.
    let out = tmp.join("tile_grid_full.jpg");
    if command_ok(
        Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-y")
            .arg("-i")
            .arg(&row.original_path)
            .arg("-frames:v")
            .arg("1")
            .arg("-q:v")
            .arg("1")
            .arg(&out),
    )
    .await
    {
        return Ok(out);
    }
    Err(Error::Internal(anyhow::anyhow!("tile-grid decode failed")))
}

async fn representative_video_frame(row: &AssetRow, tmp: &Path) -> Result<PathBuf> {
    if first_video_stream(&row.original_path).await?.is_none() {
        return Err(Error::Internal(anyhow::anyhow!("no video stream")));
    }
    let times = video_seek_times(row.duration);
    let mut best: Option<(PathBuf, f64)> = None;
    for (idx, t) in times.iter().enumerate() {
        let out = tmp.join(format!("frame_{idx}.jpg"));
        let seek = format!("{t:.3}");
        if !command_ok(
            Command::new("ffmpeg")
                .arg("-hide_banner")
                .arg("-loglevel")
                .arg("error")
                .arg("-y")
                .arg("-ss")
                .arg(seek)
                .arg("-i")
                .arg(&row.original_path)
                .arg("-frames:v")
                .arg("1")
                .arg("-vf")
                .arg("scale=1920:-2")
                .arg("-q:v")
                .arg("2")
                .arg(&out),
        )
        .await
        {
            continue;
        }
        let mean = image_mean(&out).await.unwrap_or(0.0);
        if best.as_ref().map(|(_, m)| mean > *m).unwrap_or(true) {
            best = Some((out.clone(), mean));
        }
        if mean > 3000.0 {
            break;
        }
    }
    best.map(|(path, _)| path)
        .ok_or_else(|| Error::Internal(anyhow::anyhow!("failed to extract representative frame")))
}

fn video_seek_times(duration: Option<f64>) -> Vec<f64> {
    let mut values = Vec::new();
    if let Some(d) = duration.filter(|d| d.is_finite() && *d > 0.0) {
        for v in [
            0.0,
            (0.1_f64).min(d / 3.0),
            d / 2.0,
            d * 0.8,
            1.0,
            3.0,
            5.0,
            8.0,
        ] {
            if v < d {
                values.push((v * 1000.0).round() / 1000.0);
            }
        }
    } else {
        values.extend([0.0, 1.0, 3.0, 5.0, 8.0]);
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values.dedup_by(|a, b| (*a - *b).abs() < 0.001);
    values
}

async fn first_video_stream(path: &Path) -> Result<Option<usize>> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v")
        .arg("-show_entries")
        .arg("stream=index,pix_fmt")
        .arg("-of")
        .arg("csv=p=0")
        .arg(path)
        .output()
        .await?;
    if !output.status.success() {
        return Ok(None);
    }
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.split(',');
        let Some(idx) = parts.next().and_then(|s| s.parse::<usize>().ok()) else {
            continue;
        };
        let pix_fmt = parts.next().unwrap_or("");
        if pix_fmt != "gray" && pix_fmt != "gray10le" {
            return Ok(Some(idx));
        }
    }
    Ok(None)
}

async fn image_mean(path: &Path) -> Result<f64> {
    let output = Command::new("identify")
        .arg("-format")
        .arg("%[mean]")
        .arg(path)
        .output()
        .await?;
    if !output.status.success() {
        return Ok(0.0);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0.0))
}

async fn fingerprint(path: &Path) -> Result<Fingerprint> {
    let meta = fs::metadata(path).await?;
    let modified = meta
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    Ok(Fingerprint {
        path: path.to_string_lossy().to_string(),
        size: meta.len(),
        mtime_ns: modified.as_nanos() as i128,
    })
}

async fn finish_file(path: &Path) -> Result<()> {
    let _ = Command::new("chown")
        .arg("tiger:tiger")
        .arg(path)
        .status()
        .await;
    let _ = Command::new("chmod").arg("0644").arg(path).status().await;
    Ok(())
}

async fn upsert_asset_file(asset_id: &str, file_type: &str, path: &Path) -> Result<()> {
    finish_file(path).await?;
    let sql = format!(
        r#"
INSERT INTO asset_file ("assetId", type, path, "createdAt", "updatedAt", "isEdited")
VALUES ({}::uuid, {}, {}, now(), now(), false)
ON CONFLICT ("assetId", type, "isEdited") DO UPDATE SET path = EXCLUDED.path, "updatedAt" = now()
"#,
        sql_quote(asset_id),
        sql_quote(file_type),
        sql_quote(&path.to_string_lossy())
    );
    psql_exec(&sql).await
}

async fn refresh_asset(asset_id: &str, thumb_path: Option<&Path>) -> Result<()> {
    if let Some(thumb) = thumb_path {
        if let Ok(hash) = compute_thumbhash(thumb).await {
            let sql = format!("UPDATE asset SET thumbhash = decode({}, 'base64'), \"updatedAt\" = now(), \"updateId\" = immich_uuid_v7() WHERE id = {}::uuid", sql_quote(&hash), sql_quote(asset_id));
            return psql_exec(&sql).await;
        }
    }
    let sql = format!("UPDATE asset SET \"updatedAt\" = now(), \"updateId\" = immich_uuid_v7() WHERE id = {}::uuid", sql_quote(asset_id));
    psql_exec(&sql).await
}

async fn compute_thumbhash(path: &Path) -> Result<String> {
    let script = r#"
const sharp = require('/opt/immich/server/node_modules/sharp');
const { rgbaToThumbHash } = require('/opt/immich/server/node_modules/thumbhash');
(async () => {
  const { data, info } = await sharp(process.argv[2]).resize(100, 100, { fit: 'inside' }).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
  process.stdout.write(Buffer.from(rgbaToThumbHash(info.width, info.height, data)).toString('base64'));
})().catch((err) => { console.error(err && err.stack || err); process.exit(1); });
"#;
    let node = node_bin()
        .await
        .ok_or_else(|| Error::Internal(anyhow::anyhow!("node not found")))?;
    let output = Command::new(node)
        .arg("-e")
        .arg(script)
        .arg(path)
        .output()
        .await?;
    if !output.status.success() {
        return Err(Error::Internal(anyhow::anyhow!(
            "thumbhash node helper failed"
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

async fn node_bin() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("NODE_BIN") {
        let path = PathBuf::from(value);
        if path.exists() {
            return Some(path);
        }
    }
    let candidate = PathBuf::from("/opt/src/software/tools/nvm/versions/node/v24.18.0/bin/node");
    if candidate.exists() {
        return Some(candidate);
    }
    if let Ok(output) = Command::new("sh")
        .arg("-lc")
        .arg("command -v node")
        .output()
        .await
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if !s.is_empty() {
                return Some(PathBuf::from(s));
            }
        }
    }
    None
}

async fn command_ok(cmd: &mut Command) -> bool {
    matches!(cmd.stdout(Stdio::null()).stderr(Stdio::null()).status().await, Ok(status) if status.success())
}

async fn psql_query(sql: &str) -> Result<String> {
    let output = psql_base()
        .arg("-At")
        .arg("-F")
        .arg("\t")
        .arg("-c")
        .arg(sql)
        .output()
        .await?;
    if !output.status.success() {
        return Err(Error::Database(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn psql_exec(sql: &str) -> Result<()> {
    let output = psql_base()
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-c")
        .arg(sql)
        .output()
        .await?;
    if !output.status.success() {
        return Err(Error::Database(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(())
}

fn psql_base() -> Command {
    let mut cmd = Command::new("sudo");
    cmd.arg("-u")
        .arg("postgres")
        .arg("psql")
        .arg("-d")
        .arg("immich");
    cmd
}

async fn read_checkpoint(path: &Path) -> Result<HashSet<String>> {
    let mut set = HashSet::new();
    let Ok(text) = fs::read_to_string(path).await else {
        return Ok(set);
    };
    for line in text.lines() {
        if let Some((id, _)) = line.split_once('\t') {
            set.insert(id.to_owned());
        }
    }
    Ok(set)
}

async fn append_checkpoint(path: &Path, asset_id: &str, status: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write_all(format!("{asset_id}\t{status}\n").as_bytes())
        .await?;
    Ok(())
}

async fn read_strategy_cache(path: &Path) -> Result<StrategyCache> {
    let Ok(text) = fs::read_to_string(path).await else {
        return Ok(HashMap::new());
    };
    Ok(serde_json::from_str(&text).unwrap_or_default())
}

async fn write_strategy_cache(path: &Path, cache: &StrategyCache) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let data = serde_json::to_vec_pretty(cache).map_err(|e| Error::Internal(e.into()))?;
    fs::write(path, data).await?;
    Ok(())
}
