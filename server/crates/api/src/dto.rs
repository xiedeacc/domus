//! Response DTOs. Field names and casing must match the Immich OpenAPI spec
//! (camelCase, ISO-8601 timestamps, UUID strings) — clients are generated
//! from that spec and break on any drift.

use base64::Engine;
use chrono::{DateTime, Utc};
use domus_common::types::{AssetType, AssetVisibility};
use domus_db::entities::{Album, Asset, Exif, Memory, Session, SharedLink, Stack, User};
use domus_domain::services::partner::PartnerWithUser;
use serde::Serialize;
use uuid::Uuid;

fn iso(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponseDto {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub profile_image_path: String,
    pub avatar_color: String,
    pub profile_changed_at: String,
}

impl From<&User> for UserResponseDto {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            email: u.email.clone(),
            name: u.name.clone(),
            profile_image_path: u.profile_image_path.clone(),
            avatar_color: u.avatar_color.clone().unwrap_or_else(|| "primary".into()),
            profile_changed_at: iso(&u.profile_changed_at),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAdminResponseDto {
    #[serde(flatten)]
    pub base: UserResponseDto,
    pub storage_label: Option<String>,
    pub should_change_password: bool,
    pub is_admin: bool,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub oauth_id: String,
    pub quota_size_in_bytes: Option<i64>,
    pub quota_usage_in_bytes: i64,
    pub status: String,
    pub license: Option<serde_json::Value>,
}

impl From<&User> for UserAdminResponseDto {
    fn from(u: &User) -> Self {
        Self {
            base: u.into(),
            storage_label: u.storage_label.clone(),
            should_change_password: u.should_change_password,
            is_admin: u.is_admin,
            created_at: iso(&u.created_at),
            updated_at: iso(&u.updated_at),
            deleted_at: u.deleted_at.as_ref().map(iso),
            oauth_id: u.oauth_id.clone(),
            quota_size_in_bytes: u.quota_size_in_bytes,
            quota_usage_in_bytes: u.quota_usage_in_bytes,
            status: "active".into(),
            license: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponseDto {
    pub access_token: String,
    pub user_id: Uuid,
    pub user_email: String,
    pub name: String,
    pub is_admin: bool,
    pub profile_image_path: String,
    pub should_change_password: bool,
    pub is_onboarded: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponseDto {
    pub id: Uuid,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: Option<String>,
    pub current: bool,
    pub device_type: String,
    pub device_os: String,
}

impl SessionResponseDto {
    pub fn from_session(s: &Session, current: bool) -> Self {
        Self {
            id: s.id,
            created_at: iso(&s.created_at),
            updated_at: iso(&s.updated_at),
            expires_at: s.expires_at.as_ref().map(iso),
            current,
            device_type: s.device_type.clone(),
            device_os: s.device_os.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumResponseDto {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub album_name: String,
    pub description: String,
    pub album_thumbnail_asset_id: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
    pub is_activity_enabled: bool,
    pub order: String,
    pub asset_count: i64,
    pub assets: Vec<serde_json::Value>,
    pub album_users: Vec<serde_json::Value>,
    pub shared: bool,
    pub has_shared_link: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

impl From<&Album> for AlbumResponseDto {
    fn from(a: &Album) -> Self {
        Self::from_album(a, 0, vec![])
    }
}

impl AlbumResponseDto {
    pub fn from_album(a: &Album, asset_count: i64, assets: Vec<serde_json::Value>) -> Self {
        let (start_date, end_date) = album_date_range(&assets);
        Self {
            id: a.id,
            owner_id: a.owner_id,
            album_name: a.album_name.clone(),
            description: a.description.clone(),
            album_thumbnail_asset_id: a.album_thumbnail_asset_id,
            created_at: iso(&a.created_at),
            updated_at: iso(&a.updated_at),
            is_activity_enabled: a.is_activity_enabled,
            order: a.order.clone(),
            asset_count,
            assets,
            album_users: vec![],
            shared: false,
            has_shared_link: false,
            start_date,
            end_date,
        }
    }
}

fn album_date_range(assets: &[serde_json::Value]) -> (Option<String>, Option<String>) {
    let mut dates: Vec<String> = assets
        .iter()
        .filter_map(|asset| asset.get("localDateTime").and_then(|value| value.as_str()))
        .map(str::to_owned)
        .collect();
    dates.sort();
    (dates.first().cloned(), dates.last().cloned())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifResponseDto {
    pub make: Option<String>,
    pub model: Option<String>,
    pub exif_image_width: Option<i32>,
    pub exif_image_height: Option<i32>,
    pub file_size_in_byte: Option<i64>,
    pub orientation: Option<String>,
    pub date_time_original: Option<String>,
    pub modify_date: Option<String>,
    pub time_zone: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub f_number: Option<f64>,
    pub focal_length: Option<f64>,
    pub iso: Option<i32>,
    pub exposure_time: Option<String>,
    pub lens_model: Option<String>,
    pub projection_type: Option<String>,
    pub rating: Option<i32>,
    pub fps: Option<f64>,
}

impl From<&Exif> for ExifResponseDto {
    fn from(e: &Exif) -> Self {
        Self {
            make: e.make.clone(),
            model: e.model.clone(),
            exif_image_width: e.exif_image_width,
            exif_image_height: e.exif_image_height,
            file_size_in_byte: e.file_size_in_byte,
            orientation: e.orientation.clone(),
            date_time_original: e.date_time_original.as_ref().map(iso),
            modify_date: e.modify_date.as_ref().map(iso),
            time_zone: e.time_zone.clone(),
            latitude: e.latitude,
            longitude: e.longitude,
            city: e.city.clone(),
            state: e.state.clone(),
            country: e.country.clone(),
            description: e.description.clone(),
            f_number: e.f_number,
            focal_length: e.focal_length,
            iso: e.iso,
            exposure_time: e.exposure_time.clone(),
            lens_model: e.lens_model.clone(),
            projection_type: e.projection_type.clone(),
            rating: e.rating,
            fps: e.fps,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetResponseDto {
    pub id: Uuid,
    pub device_asset_id: String,
    pub owner_id: Uuid,
    pub device_id: String,
    pub library_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
    pub original_path: String,
    pub original_file_name: String,
    pub resized: bool,
    pub thumbhash: Option<String>,
    pub file_created_at: String,
    pub file_modified_at: String,
    pub local_date_time: String,
    pub updated_at: String,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub is_trashed: bool,
    pub visibility: AssetVisibility,
    pub duration: Option<String>,
    pub live_photo_video_id: Option<Uuid>,
    pub stack_id: Option<Uuid>,
    pub exif_info: Option<ExifResponseDto>,
    pub people: Vec<serde_json::Value>,
    pub tags: Vec<serde_json::Value>,
    pub checksum: String,
}

impl AssetResponseDto {
    pub fn from_asset(asset: &Asset, exif: Option<&Exif>) -> Self {
        Self {
            id: asset.id,
            device_asset_id: asset.device_asset_id.clone(),
            owner_id: asset.owner_id,
            device_id: asset.device_id.clone(),
            library_id: asset.library_id,
            asset_type: asset.asset_type,
            original_path: asset.original_path.clone(),
            original_file_name: asset.original_file_name.clone(),
            resized: false,
            thumbhash: asset
                .thumbhash
                .as_ref()
                .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
            file_created_at: iso(&asset.file_created_at),
            file_modified_at: iso(&asset.file_modified_at),
            local_date_time: iso(&asset.local_date_time),
            updated_at: iso(&asset.updated_at),
            is_favorite: asset.is_favorite,
            is_archived: asset.visibility == AssetVisibility::Archive,
            is_trashed: asset.deleted_at.is_some(),
            visibility: asset.visibility,
            duration: asset.duration.clone(),
            live_photo_video_id: asset.live_photo_video_id,
            stack_id: asset.stack_id,
            exif_info: exif.map(ExifResponseDto::from),
            people: vec![],
            tags: vec![],
            checksum: base64::engine::general_purpose::STANDARD.encode(&asset.checksum),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryResponseDto {
    pub id: Uuid,
    pub owner_id: Uuid,
    #[serde(rename = "type")]
    pub memory_type: String,
    pub data: serde_json::Value,
    pub memory_at: String,
    pub is_saved: bool,
    pub seen_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub assets: Vec<AssetResponseDto>,
}

impl MemoryResponseDto {
    pub fn from_memory(memory: &Memory, assets: Vec<AssetResponseDto>) -> Self {
        Self {
            id: memory.id,
            owner_id: memory.owner_id,
            memory_type: memory.memory_type.clone(),
            data: memory.data.clone(),
            memory_at: iso(&memory.memory_at),
            is_saved: memory.is_saved,
            seen_at: memory.seen_at.as_ref().map(iso),
            created_at: iso(&memory.created_at),
            updated_at: iso(&memory.updated_at),
            assets,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerResponseDto {
    #[serde(flatten)]
    pub user: UserResponseDto,
    pub in_timeline: bool,
}

impl From<&PartnerWithUser> for PartnerResponseDto {
    fn from(partner: &PartnerWithUser) -> Self {
        Self {
            user: UserResponseDto::from(&partner.user),
            in_timeline: partner.partner.in_timeline,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StackResponseDto {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub primary_asset_id: Uuid,
    pub assets: Vec<AssetResponseDto>,
}

impl StackResponseDto {
    pub fn from_stack(stack: &Stack, assets: Vec<AssetResponseDto>) -> Self {
        Self {
            id: stack.id,
            owner_id: stack.owner_id,
            primary_asset_id: stack.primary_asset_id,
            assets,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedLinkResponseDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: String,
    pub slug: Option<String>,
    #[serde(rename = "type")]
    pub link_type: domus_common::types::SharedLinkType,
    pub album_id: Option<Uuid>,
    pub description: Option<String>,
    pub allow_upload: bool,
    pub allow_download: bool,
    pub show_exif: bool,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub assets: Vec<AssetResponseDto>,
}

impl SharedLinkResponseDto {
    pub fn from_link(link: &SharedLink, assets: Vec<AssetResponseDto>) -> Self {
        Self {
            id: link.id,
            user_id: link.user_id,
            key: hex::encode(&link.key),
            slug: link.slug.clone(),
            link_type: link.link_type,
            album_id: link.album_id,
            description: link.description.clone(),
            allow_upload: link.allow_upload,
            allow_download: link.allow_download,
            show_exif: link.show_exif,
            expires_at: link.expires_at.as_ref().map(iso),
            created_at: iso(&link.created_at),
            assets,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapMarkerResponseDto {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub lat: f64,
    pub lon: f64,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub file_created_at: String,
}

impl MapMarkerResponseDto {
    pub fn from_asset_exif(asset: &Asset, exif: &Exif) -> Option<Self> {
        Some(Self {
            id: asset.id,
            asset_id: asset.id,
            lat: exif.latitude?,
            lon: exif.longitude?,
            city: exif.city.clone(),
            state: exif.state.clone(),
            country: exif.country.clone(),
            file_created_at: iso(&asset.file_created_at),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domus_db::entities::Partner;
    use domus_domain::services::partner::PartnerWithUser;
    use serde_json::json;

    fn dt(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn album() -> Album {
        Album {
            id: Uuid::parse_str("10000000-0000-0000-0000-000000000001").unwrap(),
            owner_id: Uuid::parse_str("10000000-0000-0000-0000-000000000002").unwrap(),
            album_name: "Trip".to_owned(),
            description: "Summer".to_owned(),
            album_thumbnail_asset_id: None,
            is_activity_enabled: true,
            order: "desc".to_owned(),
            created_at: dt("2026-07-14T01:02:03.456Z"),
            updated_at: dt("2026-07-14T01:02:03.456Z"),
            deleted_at: None,
        }
    }

    fn user() -> User {
        User {
            id: Uuid::parse_str("10000000-0000-0000-0000-000000000003").unwrap(),
            email: "user@example.com".to_owned(),
            password: "hash".to_owned(),
            name: "User".to_owned(),
            is_admin: true,
            avatar_color: Some("primary".to_owned()),
            profile_image_path: "".to_owned(),
            storage_label: None,
            oauth_id: "".to_owned(),
            quota_size_in_bytes: None,
            quota_usage_in_bytes: 0,
            should_change_password: false,
            created_at: dt("2026-07-14T01:02:03.456Z"),
            updated_at: dt("2026-07-14T01:02:03.456Z"),
            deleted_at: None,
            profile_changed_at: dt("2026-07-14T01:02:03.456Z"),
        }
    }

    fn partner_with_user() -> PartnerWithUser {
        PartnerWithUser {
            partner: Partner {
                shared_by_id: Uuid::parse_str("10000000-0000-0000-0000-000000000011").unwrap(),
                shared_with_id: Uuid::parse_str("10000000-0000-0000-0000-000000000003").unwrap(),
                in_timeline: true,
                created_at: dt("2026-07-14T01:02:03.456Z"),
            },
            user: user(),
        }
    }

    #[test]
    fn album_response_sets_start_and_end_dates_from_assets() {
        let dto = AlbumResponseDto::from_album(
            &album(),
            2,
            vec![
                json!({"localDateTime": "2025-01-01T01:02:03.456Z"}),
                json!({"localDateTime": "2023-02-22T05:06:29.716Z"}),
            ],
        );
        let value = serde_json::to_value(dto).unwrap();
        assert_eq!(value["startDate"], "2023-02-22T05:06:29.716Z");
        assert_eq!(value["endDate"], "2025-01-01T01:02:03.456Z");
    }

    #[test]
    fn album_response_omits_start_and_end_dates_for_empty_assets() {
        let value =
            serde_json::to_value(AlbumResponseDto::from_album(&album(), 0, vec![])).unwrap();
        assert!(value.get("startDate").is_none());
        assert!(value.get("endDate").is_none());
    }

    #[test]
    fn album_response_uses_immich_camel_case_fields() {
        let value = serde_json::to_value(AlbumResponseDto::from(&album())).unwrap();
        for field in [
            "ownerId",
            "albumName",
            "albumThumbnailAssetId",
            "createdAt",
            "updatedAt",
            "isActivityEnabled",
            "assetCount",
            "albumUsers",
            "hasSharedLink",
        ] {
            assert!(value.get(field).is_some(), "{field} missing");
        }
    }

    #[test]
    fn user_admin_response_uses_immich_camel_case_fields() {
        let value = serde_json::to_value(UserAdminResponseDto::from(&user())).unwrap();
        for field in [
            "profileImagePath",
            "avatarColor",
            "profileChangedAt",
            "storageLabel",
            "shouldChangePassword",
            "isAdmin",
            "oauthId",
            "quotaSizeInBytes",
            "quotaUsageInBytes",
        ] {
            assert!(value.get(field).is_some(), "{field} missing");
        }
    }

    #[test]
    fn partner_response_uses_immich_user_shape_with_in_timeline() {
        let value = serde_json::to_value(PartnerResponseDto::from(&partner_with_user())).unwrap();

        for field in [
            "id",
            "email",
            "name",
            "profileImagePath",
            "avatarColor",
            "profileChangedAt",
            "inTimeline",
        ] {
            assert!(value.get(field).is_some(), "{field} missing");
        }

        assert!(value.get("sharedById").is_none());
        assert!(value.get("sharedWithId").is_none());
        assert!(value.get("createdAt").is_none());
    }

    #[test]
    fn login_response_uses_immich_camel_case_fields() {
        let dto = LoginResponseDto {
            access_token: "token".to_owned(),
            user_id: Uuid::parse_str("10000000-0000-0000-0000-000000000003").unwrap(),
            user_email: "user@example.com".to_owned(),
            name: "User".to_owned(),
            is_admin: true,
            profile_image_path: "".to_owned(),
            should_change_password: false,
            is_onboarded: true,
        };
        let value = serde_json::to_value(dto).unwrap();
        for field in [
            "accessToken",
            "userId",
            "userEmail",
            "isAdmin",
            "profileImagePath",
            "shouldChangePassword",
            "isOnboarded",
        ] {
            assert!(value.get(field).is_some(), "{field} missing");
        }
    }
}
