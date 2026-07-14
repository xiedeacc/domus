use super::db_err;
use crate::entities::Stack;
use domus_common::{Error, Result};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct StackRepository {
    pool: PgPool,
}

impl StackRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Stack>> {
        sqlx::query_as::<_, Stack>(
            r#"SELECT id, "ownerId" AS owner_id, "primaryAssetId" AS primary_asset_id
               FROM stack
               WHERE "ownerId" = $1
               ORDER BY id"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn get(&self, id: Uuid) -> Result<Stack> {
        sqlx::query_as::<_, Stack>(
            r#"SELECT id, "ownerId" AS owner_id, "primaryAssetId" AS primary_asset_id
               FROM stack
               WHERE id = $1"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(db_err)
    }

    pub async fn create(&self, owner_id: Uuid, asset_ids: &[Uuid]) -> Result<Stack> {
        let primary = *asset_ids
            .first()
            .ok_or_else(|| Error::BadRequest("stack requires at least one asset".into()))?;
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        let stack = sqlx::query_as::<_, Stack>(
            r#"INSERT INTO stack ("ownerId", "primaryAssetId")
               VALUES ($1, $2)
               RETURNING id, "ownerId" AS owner_id, "primaryAssetId" AS primary_asset_id"#,
        )
        .bind(owner_id)
        .bind(primary)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_err)?;
        sqlx::query(
            r#"UPDATE asset SET "stackId" = $1, "updatedAt" = now()
               WHERE id = ANY($2) AND "ownerId" = $3"#,
        )
        .bind(stack.id)
        .bind(asset_ids)
        .bind(owner_id)
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
        tx.commit().await.map_err(db_err)?;
        Ok(stack)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;
        sqlx::query(
            r#"UPDATE asset SET "stackId" = NULL, "updatedAt" = now() WHERE "stackId" = $1"#,
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
        sqlx::query(r#"DELETE FROM stack WHERE id = $1"#)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }

    pub async fn remove_asset(&self, stack_id: Uuid, asset_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"UPDATE asset SET "stackId" = NULL, "updatedAt" = now()
               WHERE "stackId" = $1 AND id = $2"#,
        )
        .bind(stack_id)
        .bind(asset_id)
        .execute(&self.pool)
        .await
        .map_err(db_err)?;
        Ok(())
    }
}
