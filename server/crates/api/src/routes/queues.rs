//! Immich-compatible routes: admin queue control.

use crate::error::ApiResult;
use crate::state::AppState;
use axum::extract::State;
use axum::routing::{delete, get};
use axum::{Json, Router};

#[rustfmt::skip]
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/queues", get(queues))
        .route("/queues/{name}", get(super::not_implemented).put(super::not_implemented))
        .route("/queues/{name}/jobs", delete(super::not_implemented).get(super::not_implemented))
}

async fn queues(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(state.services.job_admin.all_status().await?))
}

#[cfg(test)]
mod tests {
    use domus_domain::services::job_admin::JobAdminService;
    use domus_jobs::PgJobQueue;

    #[tokio::test]
    async fn queue_status_includes_immich_ml_queues() {
        let pool = sqlx::SqlitePool::connect(":memory:").await.unwrap();
        sqlx::query(
            r#"CREATE TABLE job (
                id blob PRIMARY KEY,
                queue text NOT NULL,
                name text NOT NULL,
                payload text NOT NULL DEFAULT '{}',
                status text NOT NULL DEFAULT 'waiting',
                attempts integer NOT NULL DEFAULT 0,
                maxAttempts integer NOT NULL DEFAULT 3,
                error text,
                runAt text NOT NULL DEFAULT CURRENT_TIMESTAMP,
                createdAt text NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updatedAt text NOT NULL DEFAULT CURRENT_TIMESTAMP
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();
        let queues = JobAdminService::new(PgJobQueue::new(pool))
            .all_status()
            .await
            .unwrap();
        assert!(queues.get("smartSearch").is_some());
        assert!(queues.get("faceDetection").is_some());
        assert!(queues.get("facialRecognition").is_some());
        assert!(queues.get("ocr").is_some());
    }
}
