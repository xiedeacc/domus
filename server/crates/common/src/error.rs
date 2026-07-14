//! Unified error type. The API layer converts these into the same JSON error
//! envelope Immich returns: `{ "message": ..., "error": ..., "statusCode": ... }`.

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl Error {
    /// HTTP status code used by the Immich-compatible error envelope.
    pub fn status_code(&self) -> u16 {
        match self {
            Error::BadRequest(_) => 400,
            Error::Unauthorized(_) => 401,
            Error::Forbidden(_) => 403,
            Error::NotFound(_) => 404,
            Error::Conflict(_) => 409,
            Error::NotImplemented(_) => 501,
            _ => 500,
        }
    }
}
