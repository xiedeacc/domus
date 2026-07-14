use super::db_err;
use crate::entities::Partner;
use domus_common::{Error, Result};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PartnerRepository {
    pool: PgPool,
}

impl PartnerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<Partner>> {
        sqlx::query_as::<_, PartnerRow>(
            r#"SELECT "sharedById" AS shared_by_id, "sharedWithId" AS shared_with_id,
                      "inTimeline" AS in_timeline, "createdAt" AS created_at
               FROM partner
               WHERE "sharedById" = $1 OR "sharedWithId" = $1
               ORDER BY "createdAt" DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(db_err)
    }

    pub async fn get(&self, shared_by: Uuid, shared_with: Uuid) -> Result<Option<Partner>> {
        sqlx::query_as::<_, PartnerRow>(
            r#"SELECT "sharedById" AS shared_by_id, "sharedWithId" AS shared_with_id,
                      "inTimeline" AS in_timeline, "createdAt" AS created_at
               FROM partner
               WHERE "sharedById" = $1 AND "sharedWithId" = $2"#,
        )
        .bind(shared_by)
        .bind(shared_with)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(db_err)
    }

    pub async fn create(&self, shared_by: Uuid, shared_with: Uuid) -> Result<Partner> {
        sqlx::query_as::<_, PartnerRow>(
            r#"INSERT INTO partner ("sharedById", "sharedWithId", "inTimeline")
               VALUES ($1, $2, true)
               RETURNING "sharedById" AS shared_by_id, "sharedWithId" AS shared_with_id,
                         "inTimeline" AS in_timeline, "createdAt" AS created_at"#,
        )
        .bind(shared_by)
        .bind(shared_with)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|e| {
            if is_unique_violation(&e) {
                Error::BadRequest("Partner already exists".into())
            } else {
                db_err(e)
            }
        })
    }

    pub async fn update_timeline(
        &self,
        shared_by: Uuid,
        shared_with: Uuid,
        in_timeline: bool,
    ) -> Result<Partner> {
        sqlx::query_as::<_, PartnerRow>(
            r#"UPDATE partner SET "inTimeline" = $3
               WHERE "sharedById" = $1 AND "sharedWithId" = $2
               RETURNING "sharedById" AS shared_by_id, "sharedWithId" AS shared_with_id,
                         "inTimeline" AS in_timeline, "createdAt" AS created_at"#,
        )
        .bind(shared_by)
        .bind(shared_with)
        .bind(in_timeline)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(db_err)
    }

    pub async fn remove(&self, shared_by: Uuid, shared_with: Uuid) -> Result<()> {
        sqlx::query(r#"DELETE FROM partner WHERE "sharedById" = $1 AND "sharedWithId" = $2"#)
            .bind(shared_by)
            .bind(shared_with)
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

#[derive(sqlx::FromRow)]
struct PartnerRow {
    shared_by_id: Uuid,
    shared_with_id: Uuid,
    in_timeline: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<PartnerRow> for Partner {
    fn from(row: PartnerRow) -> Self {
        Self {
            shared_by_id: row.shared_by_id,
            shared_with_id: row.shared_with_id,
            in_timeline: row.in_timeline,
            created_at: row.created_at,
        }
    }
}
