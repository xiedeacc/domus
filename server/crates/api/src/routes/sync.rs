//! /sync — delta-sync stream + checkpoint acks (mobile client protocol).

use crate::error::ApiResult;
use crate::extractors::Auth;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use domus_domain::services::sync::SyncRequestType;
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sync/stream", post(sync_stream))
        .route("/sync/ack", get(get_acks).post(send_ack).delete(delete_acks))
        // Legacy v1 sync endpoints (deprecated but still served):
        .route("/sync/full-sync", post(super::not_implemented))
        .route("/sync/delta-sync", post(super::not_implemented))
}

#[derive(Deserialize)]
struct SyncStreamDto {
    types: Vec<SyncRequestType>,
    #[serde(default)]
    reset: bool,
}

/// POST /sync/stream — responds with `application/jsonlines+json`; each line
/// is `{"type": ..., "data": ..., "ack": ...}`. The client applies lines in
/// order and periodically posts the latest ack per type to /sync/ack.
async fn sync_stream(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<SyncStreamDto>,
) -> ApiResult<Response> {
    let session_id = ctx
        .session_id
        .ok_or_else(|| domus_common::Error::BadRequest("sync requires a session".into()))?;
    let envelopes = state
        .services
        .sync
        .stream(session_id, ctx.user_id, &dto.types, dto.reset)
        .await?;
    let mut body = String::new();
    for envelope in &envelopes {
        body.push_str(&serde_json::to_string(envelope).unwrap());
        body.push('\n');
    }
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/jsonlines+json")
        .body(Body::from(body))
        .unwrap())
}

async fn get_acks(State(state): State<AppState>, Auth(ctx): Auth) -> ApiResult<Json<serde_json::Value>> {
    let session_id = ctx
        .session_id
        .ok_or_else(|| domus_common::Error::BadRequest("sync requires a session".into()))?;
    let acks = state.services.sync.get_acks(session_id).await?;
    Ok(Json(serde_json::json!(acks
        .into_iter()
        .map(|(t, ack)| serde_json::json!({ "type": t, "ack": ack }))
        .collect::<Vec<_>>())))
}

#[derive(Deserialize)]
struct SyncAckSetDto {
    acks: Vec<String>,
}

async fn send_ack(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<SyncAckSetDto>,
) -> ApiResult<StatusCode> {
    let session_id = ctx
        .session_id
        .ok_or_else(|| domus_common::Error::BadRequest("sync requires a session".into()))?;
    state.services.sync.ack(session_id, &dto.acks).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct SyncAckDeleteDto {
    #[serde(default)]
    types: Vec<String>,
}

async fn delete_acks(
    State(state): State<AppState>,
    Auth(ctx): Auth,
    Json(dto): Json<SyncAckDeleteDto>,
) -> ApiResult<StatusCode> {
    let session_id = ctx
        .session_id
        .ok_or_else(|| domus_common::Error::BadRequest("sync requires a session".into()))?;
    state.services.sync.delete_acks(session_id, &dto.types).await?;
    Ok(StatusCode::NO_CONTENT)
}
