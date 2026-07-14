//! /users + /admin/users — profile, preferences, admin management.

use crate::dto::{UserAdminResponseDto, UserResponseDto};
use crate::error::ApiResult;
use crate::extractors::{AdminAuth, Auth};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        // self-service
        .route("/users", get(list_users))
        .route("/users/me", get(get_my_user).put(super::not_implemented))
        .route("/users/me/preferences", get(super::not_implemented).put(super::not_implemented))
        .route("/users/me/license", get(super::not_implemented).put(super::not_implemented).delete(super::not_implemented))
        .route("/users/me/onboarding", get(super::not_implemented).put(super::not_implemented).delete(super::not_implemented))
        .route("/users/profile-image", post(super::not_implemented).delete(super::not_implemented))
        .route("/users/{id}", get(get_user))
        .route("/users/{id}/profile-image", get(super::not_implemented))
        // admin
        .route("/admin/users", get(admin_list_users).post(super::not_implemented))
        .route(
            "/admin/users/{id}",
            get(admin_get_user).put(super::not_implemented).delete(super::not_implemented),
        )
        .route("/admin/users/{id}/restore", post(super::not_implemented))
        .route("/admin/users/{id}/preferences", get(super::not_implemented).put(super::not_implemented))
        .route("/admin/users/{id}/statistics", get(super::not_implemented))
}

async fn list_users(State(state): State<AppState>, Auth(_): Auth) -> ApiResult<Json<Vec<UserResponseDto>>> {
    let users = state.services.user.list().await?;
    Ok(Json(users.iter().map(Into::into).collect()))
}

async fn get_my_user(State(state): State<AppState>, Auth(ctx): Auth) -> ApiResult<Json<UserAdminResponseDto>> {
    let user = state.services.user.get(ctx.user_id).await?;
    Ok(Json((&user).into()))
}

async fn get_user(
    State(state): State<AppState>,
    Auth(_): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<UserResponseDto>> {
    let user = state.services.user.get(id).await?;
    Ok(Json((&user).into()))
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
