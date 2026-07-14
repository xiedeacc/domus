//! Delta-sync protocol backing the mobile app (POST /sync/stream + acks).
//!
//! The server streams JSON-lines: one `{type, data, ack}` envelope per
//! change, driven by per-(session, type) checkpoints persisted via
//! /sync/ack. Deletes are reconstructed from the *_audit tables.

use domus_common::{Error, Result};
use domus_db::Repositories;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct SyncService {
    repos: Repositories,
}

/// Entity streams a client can subscribe to. Mirrors Immich 3.0.2's
/// SyncRequestType exactly (V1 variants deprecated there stay accepted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncRequestType {
    AlbumsV1,
    AlbumsV2,
    AlbumUsersV1,
    AlbumToAssetsV1,
    AlbumAssetsV1,
    AlbumAssetsV2,
    AlbumAssetExifsV1,
    AssetsV1,
    AssetsV2,
    AssetExifsV1,
    AssetEditsV1,
    AssetMetadataV1,
    AssetOcrV1,
    AuthUsersV1,
    MemoriesV1,
    MemoryToAssetsV1,
    PartnersV1,
    PartnerAssetsV1,
    PartnerAssetsV2,
    PartnerAssetExifsV1,
    PartnerStacksV1,
    StacksV1,
    UsersV1,
    UserMetadataV1,
    PeopleV1,
}

/// One JSON line in the response stream.
#[derive(Debug, Serialize)]
pub struct SyncEnvelope {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub data: serde_json::Value,
    /// Opaque checkpoint token the client posts back to /sync/ack.
    pub ack: String,
}

impl SyncService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    /// Produce the ordered change stream for the requested types, resuming
    /// from this session's stored checkpoints. Returns batches the API layer
    /// writes as JSON lines.
    pub async fn stream(
        &self,
        _session_id: Uuid,
        _user_id: Uuid,
        _types: &[SyncRequestType],
        _reset: bool,
    ) -> Result<Vec<SyncEnvelope>> {
        Err(Error::NotImplemented("SyncService::stream"))
    }

    pub async fn ack(&self, session_id: Uuid, acks: &[String]) -> Result<()> {
        self.repos.sync.upsert_checkpoints(session_id, acks).await
    }

    pub async fn get_acks(&self, session_id: Uuid) -> Result<Vec<(String, String)>> {
        self.repos.sync.get_checkpoints(session_id).await
    }

    pub async fn delete_acks(&self, session_id: Uuid, types: &[String]) -> Result<()> {
        self.repos.sync.delete_checkpoints(session_id, types).await
    }
}
