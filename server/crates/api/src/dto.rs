//! Response DTOs. Field names and casing must match the Immich OpenAPI spec
//! (camelCase, ISO-8601 timestamps, UUID strings) — clients are generated
//! from that spec and break on any drift.

use chrono::{DateTime, Utc};
use domus_db::entities::{Album, Session, User};
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
}

impl From<&Album> for AlbumResponseDto {
    fn from(a: &Album) -> Self {
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
            asset_count: 0,
            assets: vec![],
            album_users: vec![],
            shared: false,
            has_shared_link: false,
        }
    }
}
