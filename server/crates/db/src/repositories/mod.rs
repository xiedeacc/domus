pub mod activity;
pub mod album;
pub mod api_key;
pub mod asset;
pub mod job;
pub mod library;
pub mod memory;
pub mod notification;
pub mod partner;
pub mod search;
pub mod session;
pub mod shared_link;
pub mod stack;
pub mod sync;
pub mod system_metadata;
pub mod tag;
pub mod timeline;
pub mod user;

/// Convert sqlx errors into the shared error type.
pub(crate) fn db_err(e: sqlx::Error) -> domus_common::Error {
    match e {
        sqlx::Error::RowNotFound => domus_common::Error::NotFound("row not found".into()),
        other => domus_common::Error::Database(other.to_string()),
    }
}
