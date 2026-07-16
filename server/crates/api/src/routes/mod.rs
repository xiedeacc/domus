//! One module per Immich controller. Every route the Immich 3.0.2 OpenAPI
//! spec defines is mounted here — unimplemented ones answer 501 so client
//! integrations fail loudly instead of 404-ing.

pub mod activities;
pub mod albums;
pub mod api_keys;
pub mod assets;
pub mod auth;
pub mod download;
pub mod duplicates;
pub mod faces;
pub mod immich_derivatives;
pub mod jobs;
pub mod libraries;
pub mod map;
pub mod memories;
pub mod notifications;
pub mod oauth;
pub mod partners;
pub mod people;
pub mod queues;
pub mod search;
pub mod server;
pub mod sessions;
pub mod shared_links;
pub mod stacks;
pub mod sync;
pub mod system_config;
pub mod system_metadata;
pub mod tags;
pub mod timeline;
pub mod trash;
pub mod users;
pub mod views;

use crate::error::ApiError;
use domus_common::Error;

/// Placeholder handler for routes that exist in the protocol but are not
/// implemented yet — answers 501 with the Immich error envelope.
pub async fn not_implemented() -> ApiError {
    ApiError(Error::NotImplemented(
        "this endpoint is not implemented yet",
    ))
}
