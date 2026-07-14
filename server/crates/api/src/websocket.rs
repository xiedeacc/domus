//! Realtime channel. Immich clients connect with socket.io (engine.io v4)
//! at path `/api/socket.io`, authenticating the handshake with the same
//! credentials as REST. Domus uses `socketioxide` to speak the identical
//! protocol.
//!
//! Events emitted by the server (names are wire contract):
//!   on_upload_success, on_asset_trash, on_asset_delete, on_asset_restore,
//!   on_asset_update, on_asset_hidden, on_person_thumbnail,
//!   on_server_version, on_config_update, on_new_release, on_notification,
//!   on_session_delete, AssetUploadReadyV1 ...

use socketioxide::extract::SocketRef;
use socketioxide::{layer::SocketIoLayer, SocketIo};
use tracing::info;

/// Build the socket.io layer and its event wiring.
pub fn build() -> (SocketIoLayer, SocketIo) {
    let (layer, io) = SocketIo::builder().req_path("/api/socket.io").build_layer();

    io.ns("/", |socket: SocketRef| {
        // TODO: authenticate the handshake (cookie / bearer / api key) and
        // join the socket to its user room for targeted emits.
        info!(sid = %socket.id, "socket.io client connected");
    });

    (layer, io)
}

/// Emit an event to every session of one user.
pub fn emit_to_user(io: &SocketIo, _user_id: uuid::Uuid, event: &str, payload: serde_json::Value) {
    // TODO: room-scoped emit once handshake auth lands; broadcast for now.
    let _ = io.emit(event.to_owned(), &payload);
}
