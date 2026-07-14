//! Timeline: month buckets and per-bucket asset pages.
//!
//! Wire note: /timeline/bucket returns a *columnar* payload (parallel arrays
//! per field, see TimeBucketAssetResponseDto) — significantly smaller than an
//! array of asset objects. Bucket keys are "YYYY-MM-01" date strings.

use domus_common::Result;
use domus_db::repositories::timeline::TimeBucketQuery;
use domus_db::Repositories;
use serde::Serialize;

pub struct TimelineService {
    repos: Repositories,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBucket {
    pub time_bucket: String,
    pub count: i64,
}

impl TimelineService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn buckets(&self, query: TimeBucketQuery) -> Result<Vec<TimeBucket>> {
        let rows = self.repos.timeline.get_time_buckets(query).await?;
        Ok(rows
            .into_iter()
            .map(|(time_bucket, count)| TimeBucket { time_bucket, count })
            .collect())
    }

    /// Columnar asset payload for one bucket.
    pub async fn bucket_assets(
        &self,
        bucket: &str,
        query: TimeBucketQuery,
    ) -> Result<serde_json::Value> {
        self.repos
            .timeline
            .get_time_bucket_assets(bucket, query)
            .await
    }
}
