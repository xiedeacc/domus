use domus_common::Result;
use sqlx::PgPool;
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
        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let rows: Vec<serde_json::Value> = sqlx::query_scalar(
            r#"SELECT jsonb_build_object(
                   'id', a.id,
                   'ownerId', a."ownerId",
                   'type', a.type,
                   'originalFileName', a."originalFileName",
                   'localDateTime', to_char(a."localDateTime" AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
                   'fileCreatedAt', to_char(a."fileCreatedAt" AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'),
                   'isFavorite', a."isFavorite",
                   'isTrashed', a."deletedAt" IS NOT NULL,
                   'isArchived', a.visibility = 'archive',
                   'visibility', a.visibility,
                   'thumbhash', CASE WHEN a.thumbhash IS NULL THEN NULL ELSE encode(a.thumbhash, 'base64') END,
                   'width', e."exifImageWidth",
                   'height', e."exifImageHeight"
               )
               FROM asset a
               LEFT JOIN asset_exif e ON e."assetId" = a.id
               WHERE a."ownerId" = ANY($1)
                 AND a."deletedAt" IS NULL
                 AND ($2 = '%%' OR a."originalFileName" ILIKE $2 ESCAPE '\')
               ORDER BY a."localDateTime" DESC
               LIMIT 250"#,
        )
        .bind(user_ids)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(serde_json::json!({ "assets": { "items": rows, "total": rows.len() } }))
    }

    pub async fn suggestions(&self, user_id: Uuid, kind: &str) -> Result<Vec<String>> {
        let column = match kind {
            "city" | "cities" => "city",
            "country" | "countries" => "country",
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
