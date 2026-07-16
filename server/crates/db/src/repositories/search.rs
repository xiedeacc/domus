use crate::PgPool;
use base64::Engine;
use domus_common::Result;
use sqlx::Row;
use uuid::Uuid;

#[derive(Clone)]
pub struct SearchRepository {
    pool: PgPool,
}

impl SearchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search_metadata(
        &self,
        user_ids: &[Uuid],
        filters: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let query = filters
            .get("query")
            .or_else(|| filters.get("originalFileName"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let owner_id = user_ids.first().copied().unwrap_or_default();
        let pattern = format!(
            "%{}%",
            query
                .to_ascii_lowercase()
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        let rows = sqlx::query(
            r#"SELECT a.id, a."ownerId", a.type, a."originalFileName", a."localDateTime",
                      a."fileCreatedAt", a."isFavorite", a."deletedAt", a.visibility,
                      a.thumbhash, e."exifImageWidth", e."exifImageHeight"
               FROM asset a
               LEFT JOIN asset_exif e ON e."assetId" = a.id
               WHERE a."ownerId" = $1
                 AND a."deletedAt" IS NULL
                 AND (
                    $2 = '%%'
                    OR lower(a."originalFileName") LIKE $2 ESCAPE '\'
                    OR lower(COALESCE(e.city, '')) LIKE $2 ESCAPE '\'
                    OR lower(COALESCE(e.state, '')) LIKE $2 ESCAPE '\'
                    OR lower(COALESCE(e.country, '')) LIKE $2 ESCAPE '\'
                    OR lower(COALESCE(e.make, '')) LIKE $2 ESCAPE '\'
                    OR lower(COALESCE(e.model, '')) LIKE $2 ESCAPE '\'
                    OR lower(COALESCE(e."lensModel", '')) LIKE $2 ESCAPE '\'
                    OR substr(a."localDateTime", 1, 10) LIKE $2 ESCAPE '\'
                    OR substr(a."fileCreatedAt", 1, 10) LIKE $2 ESCAPE '\'
                 )
               ORDER BY a."localDateTime" DESC
               LIMIT 250"#,
        )
        .bind(owner_id)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        let items: Vec<_> = rows
            .iter()
            .map(|row| {
                let thumbhash: Option<Vec<u8>> = row.try_get("thumbhash").ok();
                serde_json::json!({
                    "id": row.try_get::<Uuid, _>("id").ok(),
                    "ownerId": row.try_get::<Uuid, _>("ownerId").ok(),
                    "type": row.try_get::<String, _>("type").unwrap_or_default(),
                    "originalFileName": row.try_get::<String, _>("originalFileName").unwrap_or_default(),
                    "localDateTime": row.try_get::<String, _>("localDateTime").unwrap_or_default(),
                    "fileCreatedAt": row.try_get::<String, _>("fileCreatedAt").unwrap_or_default(),
                    "isFavorite": row.try_get::<bool, _>("isFavorite").unwrap_or(false),
                    "isTrashed": row.try_get::<Option<String>, _>("deletedAt").ok().flatten().is_some(),
                    "isArchived": row.try_get::<String, _>("visibility").unwrap_or_default() == "archive",
                    "visibility": row.try_get::<String, _>("visibility").unwrap_or_default(),
                    "thumbhash": thumbhash.map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes)),
                    "width": row.try_get::<Option<i32>, _>("exifImageWidth").ok().flatten(),
                    "height": row.try_get::<Option<i32>, _>("exifImageHeight").ok().flatten(),
                })
            })
            .collect();
        Ok(serde_json::json!({ "assets": { "items": items, "total": items.len() } }))
    }

    pub async fn suggestions(&self, user_id: Uuid, kind: &str) -> Result<Vec<String>> {
        let column = match kind {
            "city" | "cities" => "city",
            "state" => "state",
            "country" | "countries" => "country",
            "camera-make" => "\"make\"",
            "camera-model" => "model",
            "camera-lens-model" => "\"lensModel\"",
            _ => "city",
        };
        let sql = format!(
            r#"SELECT DISTINCT {column} FROM asset_exif e
               JOIN asset a ON a.id = e."assetId"
               WHERE a."ownerId" = $1 AND a."deletedAt" IS NULL AND {column} IS NOT NULL
               ORDER BY {column}
               LIMIT 100"#
        );
        sqlx::query_scalar(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)
    }

    pub async fn explore(&self, user_id: Uuid) -> Result<serde_json::Value> {
        let cities = self.suggestions(user_id, "city").await?;
        Ok(serde_json::json!({ "fieldName": "city", "items": cities }))
    }
}

fn db_err(e: sqlx::Error) -> domus_common::Error {
    domus_common::Error::Database(e.to_string())
}
