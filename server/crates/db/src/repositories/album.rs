use super::db_err;
use crate::entities::Album;
use domus_common::Result;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AlbumRepository {
    pool: PgPool,
}

impl AlbumRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: Uuid) -> Result<Album> {
        sqlx::query_as::<_, Album>(ALBUM_SELECT_SQL)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(db_err)
    }

    /// Albums owned by the user plus albums shared with them.
    pub async fn list_for_user(&self, user_id: Uuid, shared: Option<bool>) -> Result<Vec<Album>> {
        let shared_clause = match shared {
            Some(true) => r#"AND a."ownerId" <> $1"#,
            Some(false) => r#"AND a."ownerId" = $1"#,
            None => "",
        };
        let sql = format!(
            r#"SELECT a.id, a."ownerId" AS owner_id, a."albumName" AS album_name,
                      a.description, a."albumThumbnailAssetId" AS album_thumbnail_asset_id,
                      a."isActivityEnabled" AS is_activity_enabled, a."order",
                      a."createdAt" AS created_at, a."updatedAt" AS updated_at,
                      a."deletedAt" AS deleted_at
               FROM album a
               WHERE a."deletedAt" IS NULL
                 AND (a."ownerId" = $1 OR EXISTS (
                     SELECT 1 FROM album_user au WHERE au."albumId" = a.id AND au."userId" = $1
                 ))
                 {shared_clause}
               ORDER BY a."createdAt" DESC"#,
        );
        sqlx::query_as::<_, Album>(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)
    }

    pub async fn create(&self, owner_id: Uuid, name: &str, description: &str) -> Result<Album> {
        sqlx::query_as::<_, Album>(
            r#"INSERT INTO album ("ownerId", "albumName", description)
               VALUES ($1, $2, $3)
               RETURNING id, "ownerId" AS owner_id, "albumName" AS album_name,
                         description, "albumThumbnailAssetId" AS album_thumbnail_asset_id,
                         "isActivityEnabled" AS is_activity_enabled, "order",
                         "createdAt" AS created_at, "updatedAt" AS updated_at,
                         "deletedAt" AS deleted_at"#,
        )
        .bind(owner_id)
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn add_assets(&self, album_id: Uuid, asset_ids: &[Uuid]) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO album_asset ("albumId", "assetId")
               SELECT $1, unnest($2::uuid[])
               ON CONFLICT DO NOTHING"#,
        )
        .bind(album_id)
        .bind(asset_ids)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }

    pub async fn asset_count(&self, album_id: Uuid) -> Result<i64> {
        sqlx::query_as(
            r#"SELECT COUNT(*)::bigint
               FROM album_asset aa
               JOIN asset a ON a.id = aa."assetId"
               WHERE aa."albumId" = $1 AND a."deletedAt" IS NULL"#,
        )
        .bind(album_id)
        .fetch_one(&self.pool)
        .await
        .map(|(count,)| count)
        .map_err(db_err)
    }

    pub async fn remove_assets(&self, album_id: Uuid, asset_ids: &[Uuid]) -> Result<()> {
        sqlx::query(r#"DELETE FROM album_asset WHERE "albumId" = $1 AND "assetId" = ANY($2)"#)
            .bind(album_id)
            .bind(asset_ids)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    pub async fn add_users(&self, album_id: Uuid, users: &[(Uuid, String)]) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for (user_id, role) in users {
            sqlx::query(
                r#"INSERT INTO album_user ("albumId", "userId", role)
                   VALUES ($1, $2, $3)
                   ON CONFLICT ("albumId", "userId") DO UPDATE SET role = EXCLUDED.role"#,
            )
            .bind(album_id)
            .bind(user_id)
            .bind(role)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }

    pub async fn update_user(&self, album_id: Uuid, user_id: Uuid, role: &str) -> Result<()> {
        sqlx::query(r#"UPDATE album_user SET role = $3 WHERE "albumId" = $1 AND "userId" = $2"#)
            .bind(album_id)
            .bind(user_id)
            .bind(role)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    pub async fn remove_user(&self, album_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query(r#"DELETE FROM album_user WHERE "albumId" = $1 AND "userId" = $2"#)
            .bind(album_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query(r#"UPDATE album SET "deletedAt" = now(), "updatedAt" = now() WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}

const ALBUM_SELECT_SQL: &str = r#"SELECT id, "ownerId" AS owner_id, "albumName" AS album_name,
       description, "albumThumbnailAssetId" AS album_thumbnail_asset_id,
       "isActivityEnabled" AS is_activity_enabled, "order",
       "createdAt" AS created_at, "updatedAt" AS updated_at, "deletedAt" AS deleted_at
FROM album WHERE id = $1 AND "deletedAt" IS NULL"#;
