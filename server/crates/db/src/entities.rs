//! Row types mapped 1:1 onto the database schema (which mirrors Immich's
//! schema so an existing Immich database can be attached directly).

use chrono::{DateTime, Utc};
use domus_common::types::{AssetType, AssetVisibility, SharedLinkType, UserAvatarColor};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub name: String,
    pub is_admin: bool,
    pub avatar_color: Option<String>,
    pub profile_image_path: String,
    pub storage_label: Option<String>,
    pub oauth_id: String,
    pub quota_size_in_bytes: Option<i64>,
    pub quota_usage_in_bytes: i64,
    pub should_change_password: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub profile_changed_at: DateTime<Utc>,
}

impl User {
    pub fn avatar_color(&self) -> UserAvatarColor {
        // Falls back to the deterministic default Immich derives from the id.
        UserAvatarColor::Primary
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub token: String, // SHA-256 hash of the bearer token
    pub user_id: Uuid,
    pub device_type: String,
    pub device_os: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key: String, // SHA-256 hash of the raw key
    pub user_id: Uuid,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub library_id: Option<Uuid>,
    pub device_asset_id: String,
    pub device_id: String,
    pub asset_type: AssetType,
    pub original_path: String,
    pub original_file_name: String,
    pub checksum: Vec<u8>, // SHA-1
    pub visibility: AssetVisibility,
    pub is_favorite: bool,
    pub is_offline: bool,
    pub is_external: bool,
    pub live_photo_video_id: Option<Uuid>,
    pub stack_id: Option<Uuid>,
    pub duration: Option<String>,
    pub file_created_at: DateTime<Utc>,
    pub file_modified_at: DateTime<Utc>,
    pub local_date_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct Exif {
    pub asset_id: Uuid,
    pub make: Option<String>,
    pub model: Option<String>,
    pub exif_image_width: Option<i32>,
    pub exif_image_height: Option<i32>,
    pub file_size_in_byte: Option<i64>,
    pub orientation: Option<String>,
    pub date_time_original: Option<DateTime<Utc>>,
    pub modify_date: Option<DateTime<Utc>>,
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Album {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub album_name: String,
    pub description: String,
    pub album_thumbnail_asset_id: Option<Uuid>,
    pub is_activity_enabled: bool,
    pub order: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct SharedLink {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: Vec<u8>,
    pub slug: Option<String>,
    pub link_type: SharedLinkType,
    pub album_id: Option<Uuid>,
    pub description: Option<String>,
    pub password: Option<String>,
    pub allow_upload: bool,
    pub allow_download: bool,
    pub show_exif: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub memory_type: String,
    pub data: serde_json::Value,
    pub memory_at: DateTime<Utc>,
    pub is_saved: bool,
    pub seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Partner {
    pub shared_by_id: Uuid,
    pub shared_with_id: Uuid,
    pub in_timeline: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub user_id: Uuid,
    pub value: String, // full hierarchical path, e.g. "Nature/Water"
    pub color: Option<String>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Stack {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub primary_asset_id: Uuid,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Library {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub import_paths: Vec<String>,
    pub exclusion_patterns: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub refreshed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub album_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub comment: Option<String>,
    pub is_liked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub level: String,
    pub notification_type: String,
    pub title: String,
    pub description: Option<String>,
    pub data: Option<serde_json::Value>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
