//! Metadata search (SQL filters). Smart/CLIP search is intentionally unsupported (no ML service) and reports as disabled in server features.

use domus_common::{Error, Result};
use domus_db::Repositories;
use uuid::Uuid;

pub struct SearchService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl SearchService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn search_metadata(
        &self,
        user_id: Uuid,
        filters: serde_json::Value,
    ) -> Result<serde_json::Value> {
        validate_metadata_filters(&filters)?;
        self.repos.search.search_metadata(&[user_id], filters).await
    }

    pub async fn suggestions(&self, user_id: Uuid, kind: &str) -> Result<Vec<String>> {
        self.repos.search.suggestions(user_id, kind).await
    }

    pub async fn explore(&self, user_id: Uuid) -> Result<serde_json::Value> {
        self.repos.search.explore(user_id).await
    }
}

pub fn validate_metadata_filters(filters: &serde_json::Value) -> Result<()> {
    validate_positive_integer(filters, "page")?;
    validate_positive_integer(filters, "size")?;
    validate_bool(filters, "isFavorite")?;
    validate_bool(filters, "isEncoded")?;
    validate_bool(filters, "isOffline")?;
    validate_bool(filters, "isMotion")?;

    if let Some(visibility) = filters.get("visibility") {
        match visibility.as_str() {
            Some("archive" | "timeline" | "hidden" | "locked") => {}
            _ => {
                return Err(Error::BadRequest(
                    "visibility must be one of archive, timeline, hidden, locked".into(),
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_random_filters(filters: &serde_json::Value) -> Result<()> {
    validate_bool(filters, "withStacked")?;
    validate_bool(filters, "withPeople")
}

pub fn parse_suggestion_type(value: &str) -> Result<&'static str> {
    match value {
        "country" => Ok("country"),
        "state" => Ok("state"),
        "city" => Ok("city"),
        "camera-make" => Ok("camera-make"),
        "camera-model" => Ok("camera-model"),
        "camera-lens-model" => Ok("camera-lens-model"),
        _ => Err(Error::BadRequest(
            "type must be one of country, state, city, camera-make, camera-model, camera-lens-model"
                .into(),
        )),
    }
}

fn validate_positive_integer(filters: &serde_json::Value, field: &str) -> Result<()> {
    let Some(value) = filters.get(field) else {
        return Ok(());
    };
    if value.as_i64().is_some_and(|n| n >= 1) {
        return Ok(());
    }
    Err(Error::BadRequest(format!(
        "{field} must be an integer >= 1"
    )))
}

fn validate_bool(filters: &serde_json::Value, field: &str) -> Result<()> {
    let Some(value) = filters.get(field) else {
        return Ok(());
    };
    if value.is_boolean() {
        return Ok(());
    }
    Err(Error::BadRequest(format!("{field} must be a boolean")))
}

#[cfg(test)]
mod tests {
    use super::{parse_suggestion_type, validate_metadata_filters, validate_random_filters};
    use serde_json::json;

    #[test]
    fn metadata_filters_reject_invalid_page_size_and_bool_types() {
        assert!(validate_metadata_filters(&json!({ "page": 0 })).is_err());
        assert!(validate_metadata_filters(&json!({ "page": "abc" })).is_err());
        assert!(validate_metadata_filters(&json!({ "size": -1 })).is_err());
        assert!(validate_metadata_filters(&json!({ "isFavorite": "immich" })).is_err());
        assert!(validate_metadata_filters(&json!({ "isEncoded": "immich" })).is_err());
        assert!(validate_metadata_filters(&json!({ "isOffline": "immich" })).is_err());
        assert!(validate_metadata_filters(&json!({ "isMotion": "immich" })).is_err());
    }

    #[test]
    fn metadata_filters_validate_visibility_like_immich() {
        assert!(validate_metadata_filters(&json!({ "visibility": "immich" })).is_err());
        assert!(validate_metadata_filters(&json!({ "visibility": "archive" })).is_ok());
        assert!(validate_metadata_filters(&json!({ "visibility": "timeline" })).is_ok());
    }

    #[test]
    fn random_filters_reject_invalid_boolean_types() {
        assert!(validate_random_filters(&json!({ "withStacked": "immich" })).is_err());
        assert!(validate_random_filters(&json!({ "withPeople": "immich" })).is_err());
        assert!(validate_random_filters(&json!({ "withPeople": true })).is_ok());
    }

    #[test]
    fn suggestion_type_matches_immich_enum_values() {
        assert_eq!(parse_suggestion_type("country").unwrap(), "country");
        assert_eq!(
            parse_suggestion_type("camera-lens-model").unwrap(),
            "camera-lens-model"
        );
        assert!(parse_suggestion_type("").is_err());
        assert!(parse_suggestion_type("kind").is_err());
    }
}
