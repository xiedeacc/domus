use super::db_err;
use crate::entities::Tag;
use domus_common::{Error, Result};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct TagRepository {
    pool: PgPool,
}

impl TagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Tag>> {
        sqlx::query_as::<_, Tag>(
            r#"SELECT id, "userId" AS user_id, value, color, "parentId" AS parent_id,
                      "createdAt" AS created_at, "updatedAt" AS updated_at
               FROM tag WHERE "userId" = $1 ORDER BY value"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn get(&self, id: Uuid) -> Result<Tag> {
        sqlx::query_as::<_, Tag>(
            r#"SELECT id, "userId" AS user_id, value, color, "parentId" AS parent_id,
                      "createdAt" AS created_at, "updatedAt" AS updated_at
               FROM tag WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn upsert_value(
        &self,
        user_id: Uuid,
        value: &str,
        parent_id: Option<Uuid>,
    ) -> Result<Tag> {
        sqlx::query_as::<_, Tag>(
            r#"INSERT INTO tag ("userId", value, "parentId")
               VALUES ($1, $2, $3)
               ON CONFLICT ("userId", value) DO UPDATE
               SET "parentId" = COALESCE(tag."parentId", EXCLUDED."parentId"), "updatedAt" = now()
               RETURNING id, "userId" AS user_id, value, color, "parentId" AS parent_id,
                         "createdAt" AS created_at, "updatedAt" AS updated_at"#,
        )
        .bind(user_id)
        .bind(value)
        .bind(parent_id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        value: &str,
        parent_id: Option<Uuid>,
        color: Option<&str>,
    ) -> Result<Tag> {
        sqlx::query_as::<_, Tag>(
            r#"INSERT INTO tag ("userId", value, "parentId", color)
               VALUES ($1, $2, $3, $4)
               RETURNING id, "userId" AS user_id, value, color, "parentId" AS parent_id,
                         "createdAt" AS created_at, "updatedAt" AS updated_at"#,
        )
        .bind(user_id)
        .bind(value)
        .bind(parent_id)
        .bind(color)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                Error::Conflict("tag already exists".into())
            } else {
                db_err(e)
            }
        })
    }

    pub async fn tag_assets(&self, tag_id: Uuid, asset_ids: &[Uuid]) -> Result<u64> {
        let result = sqlx::query(
            r#"INSERT INTO tag_asset ("tagId", "assetId")
               SELECT $1, unnest($2::uuid[])
               ON CONFLICT DO NOTHING"#,
        )
        .bind(tag_id)
        .bind(asset_ids)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(result.rows_affected())
    }

    pub async fn untag_assets(&self, tag_id: Uuid, asset_ids: &[Uuid]) -> Result<u64> {
        let result =
            sqlx::query(r#"DELETE FROM tag_asset WHERE "tagId" = $1 AND "assetId" = ANY($2)"#)
                .bind(tag_id)
                .bind(asset_ids)
                .execute(&self.pool)
                .await
                .map_err(db_err)?;
        Ok(result.rows_affected())
    }

    pub async fn update_color(&self, id: Uuid, color: Option<&str>) -> Result<Tag> {
        sqlx::query_as::<_, Tag>(
            r#"UPDATE tag SET color = $2, "updatedAt" = now() WHERE id = $1
               RETURNING id, "userId" AS user_id, value, color, "parentId" AS parent_id,
                         "createdAt" AS created_at, "updatedAt" AS updated_at"#,
        )
        .bind(id)
        .bind(color)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM tag WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .is_some_and(|db| db.code().as_deref() == Some("23505"))
}
