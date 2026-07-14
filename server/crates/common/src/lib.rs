//! Shared foundation for all Domus crates: configuration, error types and
//! common enums that mirror the Immich wire protocol.

pub mod config;
pub mod error;
pub mod types;
pub mod utils;

pub use config::Config;
pub use error::{Error, Result};
