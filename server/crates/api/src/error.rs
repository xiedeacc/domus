//! Map domain errors onto Immich's JSON error envelope:
//! `{ "message": "...", "error": "Bad Request", "statusCode": 400, "correlationId": "..." }`

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use domus_common::Error;

pub struct ApiError(pub Error);

impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        ApiError(e)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::json!({
            "message": self.0.to_string(),
            "error": status.canonical_reason().unwrap_or("Error"),
            "statusCode": status.as_u16(),
        });
        (status, Json(body)).into_response()
    }
}
