use domus_common::{Error, Result};
use sqlx::PgPool;
#[allow(unused_imports)]
use uuid::Uuid;

/// Filter set shared by /timeline/buckets and /timeline/bucket.
#[derive(Debug, Clone, Default)]
pub struct TimeBucketQuery {
    pub user_ids: Vec<Uuid>,
    pub album_id: Option<Uuid>,
    pub person_id: Option<Uuid>,
    pub tag_id: Option<Uuid>,
    pub is_favorite: Option<bool>,
    pub is_trashed: Option<bool>,
    pub visibility: Option<String>,
    pub with_partners: bool,
    pub with_stacked: bool,
    pub order_desc: bool,
}

#[derive(Clone)]
pub struct TimelineRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl TimelineRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_time_buckets(&self, _query: TimeBucketQuery) -> Result<Vec<(String, i64)>> {
        Err(Error::NotImplemented("TimelineRepository::get_time_buckets"))
    }

    pub async fn get_time_bucket_assets(&self, _bucket: &str, _query: TimeBucketQuery) -> Result<serde_json::Value> {
        Err(Error::NotImplemented("TimelineRepository::get_time_bucket_assets"))
    }
}
