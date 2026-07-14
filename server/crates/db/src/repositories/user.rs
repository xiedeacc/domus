use super::db_err;
use crate::entities::User;
use domus_common::Result;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: Uuid) -> Result<User> {
        sqlx::query_as::<_, User>(
            r#"SELECT id, email, password, name, "isAdmin" AS is_admin, "avatarColor" AS avatar_color,
                      "profileImagePath" AS profile_image_path, "storageLabel" AS storage_label,
                      "oauthId" AS oauth_id, "quotaSizeInBytes" AS quota_size_in_bytes,
                      "quotaUsageInBytes" AS quota_usage_in_bytes, "shouldChangePassword" AS should_change_password,
                      "createdAt" AS created_at, "updatedAt" AS updated_at, "deletedAt" AS deleted_at,
                      "profileChangedAt" AS profile_changed_at
               FROM "user" WHERE id = $1 AND "deletedAt" IS NULL"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<User>> {
        sqlx::query_as::<_, User>(
            r#"SELECT id, email, password, name, "isAdmin" AS is_admin, "avatarColor" AS avatar_color,
                      "profileImagePath" AS profile_image_path, "storageLabel" AS storage_label,
                      "oauthId" AS oauth_id, "quotaSizeInBytes" AS quota_size_in_bytes,
                      "quotaUsageInBytes" AS quota_usage_in_bytes, "shouldChangePassword" AS should_change_password,
                      "createdAt" AS created_at, "updatedAt" AS updated_at, "deletedAt" AS deleted_at,
                      "profileChangedAt" AS profile_changed_at
               FROM "user" WHERE email = $1 AND "deletedAt" IS NULL"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn list(&self) -> Result<Vec<User>> {
        sqlx::query_as::<_, User>(
            r#"SELECT id, email, password, name, "isAdmin" AS is_admin, "avatarColor" AS avatar_color,
                      "profileImagePath" AS profile_image_path, "storageLabel" AS storage_label,
                      "oauthId" AS oauth_id, "quotaSizeInBytes" AS quota_size_in_bytes,
                      "quotaUsageInBytes" AS quota_usage_in_bytes, "shouldChangePassword" AS should_change_password,
                      "createdAt" AS created_at, "updatedAt" AS updated_at, "deletedAt" AS deleted_at,
                      "profileChangedAt" AS profile_changed_at
               FROM "user" WHERE "deletedAt" IS NULL ORDER BY "createdAt""#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn count_admins(&self) -> Result<i64> {
        let (count,): (i64,) =
            sqlx::query_as(r#"SELECT COUNT(*) FROM "user" WHERE "isAdmin" AND "deletedAt" IS NULL"#)
                .fetch_one(&self.pool)
                .await
                .map_err(db_err)?;
        Ok(count)
    }

    pub async fn create(&self, email: &str, hashed_password: &str, name: &str, is_admin: bool) -> Result<User> {
        sqlx::query_as::<_, User>(
            r#"INSERT INTO "user" (email, password, name, "isAdmin")
               VALUES ($1, $2, $3, $4)
               RETURNING id, email, password, name, "isAdmin" AS is_admin, "avatarColor" AS avatar_color,
                      "profileImagePath" AS profile_image_path, "storageLabel" AS storage_label,
                      "oauthId" AS oauth_id, "quotaSizeInBytes" AS quota_size_in_bytes,
                      "quotaUsageInBytes" AS quota_usage_in_bytes, "shouldChangePassword" AS should_change_password,
                      "createdAt" AS created_at, "updatedAt" AS updated_at, "deletedAt" AS deleted_at,
                      "profileChangedAt" AS profile_changed_at"#,
        )
        .bind(email)
        .bind(hashed_password)
        .bind(name)
        .bind(is_admin)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)
    }
}
