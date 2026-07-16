//! /users + /admin/users — profile, preferences, admin management.

use crate::dto::{UserAdminResponseDto, UserResponseDto};
use crate::error::ApiResult;
use crate::extractors::{AdminAuth, Auth};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        // self-service
        .route("/users", get(list_users))
        .route("/users/me", get(get_my_user).put(super::not_implemented))
        .route(
            "/users/me/preferences",
            get(get_my_preferences).put(update_my_preferences),
        )
        .route(
            "/users/me/license",
            get(get_my_license)
                .put(update_my_license)
                .delete(delete_my_license),
        )
        .route(
            "/users/me/onboarding",
            get(get_my_onboarding)
                .put(update_my_onboarding)
                .delete(delete_my_onboarding),
        )
        .route(
            "/users/profile-image",
            post(super::not_implemented).delete(super::not_implemented),
        )
        .route("/users/{id}", get(get_user))
        .route("/users/{id}/profile-image", get(super::not_implemented))
        // admin
        .route(
            "/admin/users",
            get(admin_list_users).post(admin_create_user),
        )
        .route(
            "/admin/users/{id}",
            get(admin_get_user)
                .put(super::not_implemented)
                .delete(super::not_implemented),
        )
        .route("/admin/users/{id}/restore", post(super::not_implemented))
        .route(
            "/admin/users/{id}/preferences",
            get(super::not_implemented).put(super::not_implemented),
        )
        .route("/admin/users/{id}/statistics", get(super::not_implemented))
}

async fn list_users(
    State(state): State<AppState>,
    Auth(_): Auth,
) -> ApiResult<Json<Vec<UserResponseDto>>> {
    let users = state.services.user.list().await?;
    Ok(Json(users.iter().map(Into::into).collect()))
}

async fn get_my_user(
    State(state): State<AppState>,
    Auth(ctx): Auth,
) -> ApiResult<Json<UserAdminResponseDto>> {
    let user = state.services.user.get(ctx.user_id).await?;
    Ok(Json((&user).into()))
}

async fn get_my_preferences(Auth(_): Auth) -> Json<serde_json::Value> {
    Json(default_user_preferences())
}

async fn update_my_preferences(
    Auth(_): Auth,
    Json(dto): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut preferences = default_user_preferences();
    merge_json(&mut preferences, dto);
    Json(preferences)
}

async fn get_my_license(Auth(_): Auth) -> Json<serde_json::Value> {
    Json(serde_json::json!(null))
}

async fn update_my_license(
    Auth(_): Auth,
    Json(_dto): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!(null))
}

async fn delete_my_license(Auth(_): Auth) -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn get_my_onboarding(Auth(_): Auth) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "isOnboarded": true }))
}

async fn update_my_onboarding(
    Auth(_): Auth,
    Json(dto): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "isOnboarded": dto
            .get("isOnboarded")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true)
    }))
}

async fn delete_my_onboarding(Auth(_): Auth) -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn get_user(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<UserResponseDto>> {
    let user = state.services.user.get(id).await?;
    Ok(Json((&user).into()))
}

fn default_user_preferences() -> serde_json::Value {
    serde_json::json!({
        "albums": { "defaultAssetOrder": "desc" },
        "cast": { "gCastEnabled": false },
        "download": {
            "archiveSize": 4_294_967_296_i64,
            "includeEmbeddedVideos": true
        },
        "emailNotifications": {
            "albumInvite": true,
            "albumUpdate": true,
            "enabled": false
        },
        "folders": { "enabled": true, "sidebarWeb": true },
        "memories": { "duration": 3, "enabled": true },
        "people": { "enabled": false, "minimumFaces": 3, "sidebarWeb": false },
        "purchase": { "hideBuyButtonUntil": "", "showSupportBadge": false },
        "ratings": { "enabled": true },
        "recentlyAdded": { "sidebarWeb": true },
        "sharedLinks": { "enabled": true, "sidebarWeb": true },
        "tags": { "enabled": true, "sidebarWeb": true }
    })
}

fn merge_json(target: &mut serde_json::Value, patch: serde_json::Value) {
    match (target, patch) {
        (serde_json::Value::Object(target), serde_json::Value::Object(patch)) => {
            for (key, value) in patch {
                merge_json(target.entry(key).or_insert(serde_json::Value::Null), value);
            }
        }
        (target, value) => *target = value,
    }
}

#[cfg(test)]
mod tests {
    use super::{default_user_preferences, merge_json};
    use serde_json::json;

    #[test]
    fn user_preferences_include_all_immich_required_sections() {
        let preferences = default_user_preferences();
        for key in [
            "albums",
            "cast",
            "download",
            "emailNotifications",
            "folders",
            "memories",
            "people",
            "purchase",
            "ratings",
            "recentlyAdded",
            "sharedLinks",
            "tags",
        ] {
            assert!(preferences.get(key).is_some(), "missing {key}");
        }
        assert_eq!(preferences["memories"]["enabled"], true);
    }

    #[test]
    fn user_preferences_update_merges_nested_values() {
        let mut preferences = default_user_preferences();
        merge_json(&mut preferences, json!({"memories": {"enabled": false}}));
        assert_eq!(preferences["memories"]["enabled"], false);
        assert_eq!(preferences["memories"]["duration"], 3);
    }
}

async fn admin_list_users(
    State(state): State<AppState>,
    AdminAuth(_): AdminAuth,
) -> ApiResult<Json<Vec<UserAdminResponseDto>>> {
    let users = state.services.user.list().await?;
    Ok(Json(users.iter().map(Into::into).collect()))
}

async fn admin_get_user(
    State(state): State<AppState>,
    AdminAuth(_): AdminAuth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<UserAdminResponseDto>> {
    let user = state.services.user.get(id).await?;
    Ok(Json((&user).into()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminCreateUserDto {
    email: String,
    password: String,
    name: String,
    #[serde(default)]
    is_admin: bool,
}

async fn admin_create_user(
    State(state): State<AppState>,
    AdminAuth(_): AdminAuth,
    Json(dto): Json<AdminCreateUserDto>,
) -> ApiResult<(StatusCode, Json<UserAdminResponseDto>)> {
    let user = state
        .services
        .user
        .create_admin_user(&dto.email, &dto.password, &dto.name, dto.is_admin)
        .await?;
    Ok((StatusCode::CREATED, Json((&user).into())))
}
