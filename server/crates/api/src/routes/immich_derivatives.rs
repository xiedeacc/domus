use crate::error::{ApiError, ApiResult};
use crate::extractors::AdminAuth;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use domus_common::Error;
use domus_media::immich_derivatives::{
    run_repair, ImmichDerivativeProgress, ImmichDerivativeRequest, ImmichDerivativeSummary,
    RepairProgress,
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/immich-derivatives/run", post(run))
        .route("/admin/immich-derivatives/status", get(status))
        .route("/admin/immich-derivatives/cancel", post(cancel))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichDerivativeJobStatus {
    pub running: bool,
    pub job_id: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub progress: ImmichDerivativeProgress,
    pub summary: Option<ImmichDerivativeSummary>,
    pub error: Option<String>,
}

#[derive(Default)]
struct JobState {
    running: bool,
    job_id: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    progress: Option<RepairProgress>,
    summary: Option<ImmichDerivativeSummary>,
    error: Option<String>,
    cancel: Option<Arc<AtomicBool>>,
}

static JOB: OnceLock<Arc<Mutex<JobState>>> = OnceLock::new();

fn job() -> &'static Arc<Mutex<JobState>> {
    JOB.get_or_init(|| Arc::new(Mutex::new(JobState::default())))
}

async fn run(
    AdminAuth(_): AdminAuth,
    State(_state): State<AppState>,
    Json(request): Json<ImmichDerivativeRequest>,
) -> ApiResult<Json<ImmichDerivativeJobStatus>> {
    let progress = RepairProgress::new();
    let cancel = Arc::new(AtomicBool::new(false));
    let job_id = Uuid::new_v4().to_string();
    {
        let mut guard = job().lock().await;
        if guard.running {
            return Err(ApiError(Error::Conflict(
                "Immich derivative repair is already running".to_owned(),
            )));
        }
        guard.running = true;
        guard.job_id = Some(job_id.clone());
        guard.started_at = Some(chrono::Utc::now().to_rfc3339());
        guard.finished_at = None;
        guard.progress = Some(progress.clone());
        guard.summary = None;
        guard.error = None;
        guard.cancel = Some(cancel.clone());
    }

    let state = job().clone();
    tokio::spawn(async move {
        let result = run_repair(request, progress, cancel).await;
        let mut guard = state.lock().await;
        guard.running = false;
        guard.finished_at = Some(chrono::Utc::now().to_rfc3339());
        guard.cancel = None;
        match result {
            Ok(summary) => guard.summary = Some(summary),
            Err(err) => guard.error = Some(err.to_string()),
        }
    });

    status_inner().await.map(Json)
}

async fn status(AdminAuth(_): AdminAuth) -> ApiResult<Json<ImmichDerivativeJobStatus>> {
    status_inner().await.map(Json)
}

async fn cancel(AdminAuth(_): AdminAuth) -> ApiResult<Json<ImmichDerivativeJobStatus>> {
    if let Some(cancel) = job().lock().await.cancel.clone() {
        cancel.store(true, Ordering::Relaxed);
    }
    status_inner().await.map(Json)
}

async fn status_inner() -> ApiResult<ImmichDerivativeJobStatus> {
    let snapshot = {
        let guard = job().lock().await;
        let progress = guard.progress.clone();
        (
            guard.running,
            guard.job_id.clone(),
            guard.started_at.clone(),
            guard.finished_at.clone(),
            progress,
            guard.summary.clone(),
            guard.error.clone(),
        )
    };
    let progress = if let Some(progress) = snapshot.4 {
        progress.snapshot().await
    } else {
        ImmichDerivativeProgress::default()
    };
    Ok(ImmichDerivativeJobStatus {
        running: snapshot.0,
        job_id: snapshot.1,
        started_at: snapshot.2,
        finished_at: snapshot.3,
        progress,
        summary: snapshot.5,
        error: snapshot.6,
    })
}
