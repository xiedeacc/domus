use super::db_err;
use crate::entities::SharedLink;
use crate::PgPool;
use domus_common::types::SharedLinkType;
use domus_common::Result;
use sqlx::Row;
use uuid::Uuid;

#[derive(Clone)]
pub struct SharedLinkRepository {
    pool: PgPool,
}

impl SharedLinkRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_key(&self, key: &[u8]) -> Result<Option<SharedLink>> {
        let row = sqlx::query(SHARED_LINK_SELECT_SQL)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        row.map(|r| shared_link_from_row(&r)).transpose()
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<SharedLink>> {
        let row = sqlx::query(SHARED_LINK_SELECT_BY_SLUG_SQL)
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?;
        row.map(|r| shared_link_from_row(&r)).transpose()
    }

    pub async fn get(&self, id: Uuid) -> Result<SharedLink> {
        let row = sqlx::query(SHARED_LINK_SELECT_BY_ID_SQL)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(db_err)?;
        shared_link_from_row(&row)
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<SharedLink>> {
        let rows = sqlx::query(SHARED_LINK_LIST_SQL)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?;
        rows.iter().map(shared_link_from_row).collect()
    }

    pub async fn create(&self, link: SharedLink, asset_ids: &[Uuid]) -> Result<SharedLink> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        let row = sqlx::query(
            r#"INSERT INTO shared_link (
                   id, "userId", key, slug, type, "albumId", description, password,
                   "allowUpload", "allowDownload", "showExif", "expiresAt"
               )
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING id, "userId", key, slug, type, "albumId", description, password,
                         "allowUpload", "allowDownload", "showExif", "expiresAt", "createdAt""#,
        )
        .bind(link.id)
        .bind(link.user_id)
        .bind(&link.key)
        .bind(&link.slug)
        .bind(shared_link_type_to_db(link.link_type))
        .bind(link.album_id)
        .bind(&link.description)
        .bind(&link.password)
        .bind(link.allow_upload)
        .bind(link.allow_download)
        .bind(link.show_exif)
        .bind(link.expires_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err)?;
        for asset_id in asset_ids {
            sqlx::query(
                r#"INSERT INTO shared_link_asset ("sharedLinkId", "assetId")
                   VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
            )
            .bind(link.id)
            .bind(asset_id)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        shared_link_from_row(&row)
    }

    pub async fn update_options(
        &self,
        id: Uuid,
        allow_upload: Option<bool>,
        allow_download: Option<bool>,
        show_exif: Option<bool>,
        description: Option<&str>,
        password: Option<&str>,
        slug: Option<&str>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<SharedLink> {
        let row = sqlx::query(
            r#"UPDATE shared_link
               SET "allowUpload" = COALESCE($2, "allowUpload"),
                   "allowDownload" = COALESCE($3, "allowDownload"),
                   "showExif" = COALESCE($4, "showExif"),
                   description = COALESCE($5, description),
                   password = COALESCE($6, password),
                   slug = COALESCE($7, slug),
                   "expiresAt" = COALESCE($8, "expiresAt")
               WHERE id = $1
               RETURNING id, "userId", key, slug, type, "albumId", description, password,
                         "allowUpload", "allowDownload", "showExif", "expiresAt", "createdAt""#,
        )
        .bind(id)
        .bind(allow_upload)
        .bind(allow_download)
        .bind(show_exif)
        .bind(description)
        .bind(password)
        .bind(slug)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)?;
        shared_link_from_row(&row)
    }

    pub async fn asset_ids(&self, id: Uuid) -> Result<Vec<Uuid>> {
        sqlx::query_scalar(
            r#"SELECT "assetId" FROM shared_link_asset WHERE "sharedLinkId" = $1 ORDER BY "assetId""#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn add_assets(&self, id: Uuid, asset_ids: &[Uuid]) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for asset_id in asset_ids {
            sqlx::query(
                r#"INSERT INTO shared_link_asset ("sharedLinkId", "assetId")
                   VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
            )
            .bind(id)
            .bind(asset_id)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }

    pub async fn remove_assets(&self, id: Uuid, asset_ids: &[Uuid]) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        for asset_id in asset_ids {
            sqlx::query(
                r#"DELETE FROM shared_link_asset
                   WHERE "sharedLinkId" = $1 AND "assetId" = $2"#,
            )
            .bind(id)
            .bind(asset_id)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query(r#"DELETE FROM shared_link WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}

const SHARED_LINK_SELECT_SQL: &str = r#"SELECT id, "userId", key, slug, type, "albumId",
       description, password, "allowUpload", "allowDownload", "showExif", "expiresAt", "createdAt"
FROM shared_link WHERE key = $1"#;

const SHARED_LINK_SELECT_BY_SLUG_SQL: &str = r#"SELECT id, "userId", key, slug, type, "albumId",
       description, password, "allowUpload", "allowDownload", "showExif", "expiresAt", "createdAt"
FROM shared_link WHERE slug = $1"#;

const SHARED_LINK_SELECT_BY_ID_SQL: &str = r#"SELECT id, "userId", key, slug, type, "albumId",
       description, password, "allowUpload", "allowDownload", "showExif", "expiresAt", "createdAt"
FROM shared_link WHERE id = $1"#;

const SHARED_LINK_LIST_SQL: &str = r#"SELECT id, "userId", key, slug, type, "albumId",
       description, password, "allowUpload", "allowDownload", "showExif", "expiresAt", "createdAt"
FROM shared_link WHERE "userId" = $1 ORDER BY "createdAt" DESC"#;

fn shared_link_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<SharedLink> {
    Ok(SharedLink {
        id: row.try_get("id").map_err(db_err)?,
        user_id: row.try_get("userId").map_err(db_err)?,
        key: row.try_get("key").map_err(db_err)?,
        slug: row.try_get("slug").map_err(db_err)?,
        link_type: shared_link_type_from_db(
            row.try_get::<String, _>("type").map_err(db_err)?.as_str(),
        ),
        album_id: row.try_get("albumId").map_err(db_err)?,
        description: row.try_get("description").map_err(db_err)?,
        password: row.try_get("password").map_err(db_err)?,
        allow_upload: row.try_get("allowUpload").map_err(db_err)?,
        allow_download: row.try_get("allowDownload").map_err(db_err)?,
        show_exif: row.try_get("showExif").map_err(db_err)?,
        expires_at: row.try_get("expiresAt").map_err(db_err)?,
        created_at: row.try_get("createdAt").map_err(db_err)?,
    })
}

fn shared_link_type_from_db(value: &str) -> SharedLinkType {
    match value {
        "ALBUM" | "album" => SharedLinkType::Album,
        _ => SharedLinkType::Individual,
    }
}

fn shared_link_type_to_db(value: SharedLinkType) -> &'static str {
    match value {
        SharedLinkType::Album => "ALBUM",
        SharedLinkType::Individual => "INDIVIDUAL",
    }
}
