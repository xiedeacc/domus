//! Timeline: month buckets and per-bucket asset pages.
//!
//! Wire note: /timeline/bucket returns a *columnar* payload (parallel arrays
//! per field, see TimeBucketAssetResponseDto) — significantly smaller than an
//! array of asset objects. Bucket keys are "YYYY-MM-01" date strings.

use domus_common::{Error, Result};
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
        validate_with_partners(&query)?;
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
        validate_with_partners(&query)?;
        self.repos
            .timeline
            .get_time_bucket_assets(bucket, query)
            .await
    }
}

fn validate_with_partners(query: &TimeBucketQuery) -> Result<()> {
    if !query.with_partners {
        return Ok(());
    }
    if query.visibility.as_deref().is_some_and(|v| v != "timeline")
        || query.is_favorite.is_some()
        || query.is_trashed == Some(true)
    {
        return Err(Error::BadRequest(
            "withPartners can only be used with timeline assets".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_with_partners;
    use domus_db::repositories::timeline::TimeBucketQuery;

    #[test]
    fn with_partners_rejects_immich_incompatible_filters() {
        assert!(validate_with_partners(&TimeBucketQuery {
            with_partners: true,
            visibility: Some("archive".to_owned()),
            ..Default::default()
        })
        .is_err());
        assert!(validate_with_partners(&TimeBucketQuery {
            with_partners: true,
            visibility: Some("locked".to_owned()),
            ..Default::default()
        })
        .is_err());
        assert!(validate_with_partners(&TimeBucketQuery {
            with_partners: true,
            is_favorite: Some(false),
            ..Default::default()
        })
        .is_err());
        assert!(validate_with_partners(&TimeBucketQuery {
            with_partners: true,
            is_trashed: Some(true),
            ..Default::default()
        })
        .is_err());
    }

    #[test]
    fn with_partners_allows_timeline_filter() {
        assert!(validate_with_partners(&TimeBucketQuery {
            with_partners: true,
            visibility: Some("timeline".to_owned()),
            ..Default::default()
        })
        .is_ok());
    }
}
