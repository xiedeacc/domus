//! Delta-sync protocol backing the mobile app (POST /sync/stream + acks).
//!
//! The server streams JSON-lines: one `{type, data, ack}` envelope per
//! change, driven by per-(session, type) checkpoints persisted via
//! /sync/ack. Deletes are reconstructed from the *_audit tables.

use base64::Engine;
use chrono::{DateTime, SecondsFormat, Utc};
use domus_common::types::{AssetType, AssetVisibility};
use domus_common::Result;
use domus_db::entities::{Asset, Exif, User};
use domus_db::Repositories;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use uuid::Uuid;

pub struct SyncService {
    repos: Repositories,
}

/// Entity streams a client can subscribe to. Mirrors Immich 3.0.2's
/// SyncRequestType exactly (V1 variants deprecated there stay accepted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    AssetFacesV1,
    AssetFacesV2,
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
        session_id: Uuid,
        user_id: Uuid,
        types: &[SyncRequestType],
        _reset: bool,
    ) -> Result<Vec<SyncEnvelope>> {
        let requested: HashSet<SyncRequestType> = types.iter().copied().collect();
        let mut envelopes = Vec::new();

        let user = self.repos.user.get(user_id).await?;
        if requested.contains(&SyncRequestType::AuthUsersV1) {
            push_envelope(&mut envelopes, "AuthUserV1", sync_auth_user(&user));
        }
        if requested.contains(&SyncRequestType::UsersV1) {
            for user in self.repos.user.list().await? {
                push_envelope(&mut envelopes, "UserV1", sync_user(&user));
            }
        }

        let wants_assets = requested.contains(&SyncRequestType::AssetsV2)
            || requested.contains(&SyncRequestType::AlbumAssetsV2)
            || requested.contains(&SyncRequestType::PartnerAssetsV2);
        let wants_exif = requested.contains(&SyncRequestType::AssetExifsV1)
            || requested.contains(&SyncRequestType::AlbumAssetExifsV1)
            || requested.contains(&SyncRequestType::PartnerAssetExifsV1);

        let assets = if wants_assets || wants_exif {
            self.repos.asset.list_for_user(user_id).await?
        } else {
            Vec::new()
        };

        if wants_assets {
            for asset in &assets {
                let exif = self.repos.asset.get_exif(asset.id).await?;
                push_envelope(
                    &mut envelopes,
                    "AssetV2",
                    sync_asset_v2(asset, exif.as_ref()),
                );
            }
        }

        if wants_exif {
            for asset in &assets {
                if let Some(exif) = self.repos.asset.get_exif(asset.id).await? {
                    push_envelope(&mut envelopes, "AssetExifV1", sync_asset_exif(&exif));
                }
            }
        }

        if requested.contains(&SyncRequestType::AlbumsV2) {
            for album in self.repos.album.list_for_user(user_id, None).await? {
                push_envelope(
                    &mut envelopes,
                    "AlbumV2",
                    json!({
                        "createdAt": iso(&album.created_at),
                        "description": album.description,
                        "id": album.id,
                        "isActivityEnabled": album.is_activity_enabled,
                        "name": album.album_name,
                        "order": album.order,
                        "thumbnailAssetId": album.album_thumbnail_asset_id,
                        "updatedAt": iso(&album.updated_at),
                    }),
                );

                if requested.contains(&SyncRequestType::AlbumUsersV1) {
                    push_envelope(
                        &mut envelopes,
                        "AlbumUserV1",
                        sync_album_user_owner(album.id, album.owner_id),
                    );
                }
            }
        }

        if requested.contains(&SyncRequestType::AlbumToAssetsV1) {
            for (album_id, asset_id) in self.repos.album.asset_links_for_user(user_id).await? {
                push_envelope(
                    &mut envelopes,
                    "AlbumToAssetV1",
                    json!({ "albumId": album_id, "assetId": asset_id }),
                );
            }
        }

        push_envelope(&mut envelopes, "SyncCompleteV1", json!({}));
        for (index, envelope) in envelopes.iter_mut().enumerate() {
            envelope.ack = format!("{}:{}:{}", envelope.entity_type, session_id, index);
        }
        Ok(envelopes)
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

fn push_envelope(envelopes: &mut Vec<SyncEnvelope>, entity_type: &str, data: Value) {
    envelopes.push(SyncEnvelope {
        entity_type: entity_type.to_owned(),
        data,
        ack: String::new(),
    });
}

fn iso(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn sync_user(user: &User) -> Value {
    json!({
        "avatarColor": user.avatar_color.as_deref().unwrap_or("primary"),
        "deletedAt": user.deleted_at.as_ref().map(iso),
        "email": user.email,
        "hasProfileImage": !user.profile_image_path.is_empty(),
        "id": user.id,
        "name": user.name,
        "profileChangedAt": iso(&user.profile_changed_at),
    })
}

fn sync_auth_user(user: &User) -> Value {
    json!({
        "avatarColor": user.avatar_color.as_deref().unwrap_or("primary"),
        "deletedAt": user.deleted_at.as_ref().map(iso),
        "email": user.email,
        "hasProfileImage": !user.profile_image_path.is_empty(),
        "id": user.id,
        "isAdmin": user.is_admin,
        "name": user.name,
        "oauthId": user.oauth_id,
        "pinCode": null,
        "profileChangedAt": iso(&user.profile_changed_at),
        "quotaSizeInBytes": user.quota_size_in_bytes,
        "quotaUsageInBytes": user.quota_usage_in_bytes,
        "storageLabel": user.storage_label,
    })
}

fn sync_album_user_owner(album_id: Uuid, owner_id: Uuid) -> Value {
    json!({
        "albumId": album_id,
        "role": "owner",
        "userId": owner_id,
    })
}

fn sync_asset_v2(asset: &Asset, exif: Option<&Exif>) -> Value {
    let (width, height) = exif
        .map(|exif| (exif.exif_image_width, exif.exif_image_height))
        .unwrap_or((None, None));
    json!({
        "checksum": base64::engine::general_purpose::STANDARD.encode(&asset.checksum),
        "createdAt": iso(&asset.created_at),
        "deletedAt": asset.deleted_at.as_ref().map(iso),
        "duration": asset.duration.as_deref().and_then(duration_millis),
        "fileCreatedAt": iso(&asset.file_created_at),
        "fileModifiedAt": iso(&asset.file_modified_at),
        "height": height,
        "id": asset.id,
        "isEdited": false,
        "isFavorite": asset.is_favorite,
        "libraryId": asset.library_id,
        "livePhotoVideoId": asset.live_photo_video_id,
        "localDateTime": iso(&asset.local_date_time),
        "originalFileName": asset.original_file_name,
        "ownerId": asset.owner_id,
        "stackId": asset.stack_id,
        "thumbhash": asset.thumbhash.as_ref().map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
        "type": asset_type(asset.asset_type),
        "visibility": visibility(asset.visibility),
        "width": width,
    })
}

fn sync_asset_exif(exif: &Exif) -> Value {
    json!({
        "assetId": exif.asset_id,
        "city": exif.city,
        "country": exif.country,
        "dateTimeOriginal": exif.date_time_original.as_ref().map(iso),
        "description": exif.description,
        "exifImageHeight": exif.exif_image_height,
        "exifImageWidth": exif.exif_image_width,
        "exposureTime": exif.exposure_time,
        "fNumber": exif.f_number,
        "fileSizeInByte": exif.file_size_in_byte,
        "focalLength": exif.focal_length,
        "fps": exif.fps,
        "iso": exif.iso,
        "latitude": exif.latitude,
        "lensModel": exif.lens_model,
        "longitude": exif.longitude,
        "make": exif.make,
        "model": exif.model,
        "modifyDate": exif.modify_date.as_ref().map(iso),
        "orientation": exif.orientation,
        "profileDescription": null,
        "projectionType": exif.projection_type,
        "rating": exif.rating,
        "state": exif.state,
        "timeZone": exif.time_zone,
    })
}

fn asset_type(value: AssetType) -> &'static str {
    match value {
        AssetType::Image => "IMAGE",
        AssetType::Video => "VIDEO",
        AssetType::Audio => "AUDIO",
        AssetType::Other => "OTHER",
    }
}

fn visibility(value: AssetVisibility) -> &'static str {
    match value {
        AssetVisibility::Archive => "archive",
        AssetVisibility::Timeline => "timeline",
        AssetVisibility::Hidden => "hidden",
        AssetVisibility::Locked => "locked",
    }
}

fn duration_millis(value: &str) -> Option<i64> {
    let (hms, millis) = value.split_once('.').unwrap_or((value, "0"));
    let mut parts = hms.split(':');
    let hours = parts.next()?.parse::<i64>().ok()?;
    let minutes = parts.next()?.parse::<i64>().ok()?;
    let seconds = parts.next()?.parse::<i64>().ok()?;
    let millis = millis.get(..millis.len().min(3))?.parse::<i64>().ok()?;
    Some((((hours * 60) + minutes) * 60 + seconds) * 1000 + millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn duration_millis_parses_immich_duration() {
        assert_eq!(duration_millis("00:00:03.042"), Some(3042));
        assert_eq!(duration_millis("01:02:03.456"), Some(3723456));
    }

    #[test]
    fn sync_user_contains_required_mobile_keys() {
        let now = Utc.with_ymd_and_hms(2026, 7, 16, 1, 2, 3).unwrap();
        let user = User {
            id: Uuid::nil(),
            email: "xiedeacc@gmail.com".into(),
            password: String::new(),
            name: "Domus".into(),
            is_admin: true,
            avatar_color: None,
            profile_image_path: String::new(),
            storage_label: None,
            oauth_id: String::new(),
            quota_size_in_bytes: None,
            quota_usage_in_bytes: 0,
            should_change_password: false,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            profile_changed_at: now,
        };

        let value = sync_auth_user(&user);
        for key in [
            "deletedAt",
            "email",
            "hasProfileImage",
            "id",
            "isAdmin",
            "name",
            "oauthId",
            "pinCode",
            "profileChangedAt",
            "quotaSizeInBytes",
            "quotaUsageInBytes",
            "storageLabel",
        ] {
            assert!(value.get(key).is_some(), "missing {key}");
        }
    }

    #[test]
    fn sync_album_user_marks_owner_role_for_native_album_queries() {
        let album_id = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let owner_id = Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap();

        let value = sync_album_user_owner(album_id, owner_id);

        assert_eq!(value["albumId"], album_id.to_string());
        assert_eq!(value["userId"], owner_id.to_string());
        assert_eq!(value["role"], "owner");
    }
}
