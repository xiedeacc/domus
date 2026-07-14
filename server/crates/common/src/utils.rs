//! Small protocol helpers ported from Immich's `server/src/utils` tests.

use chrono::Datelike;
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateAsset {
    pub id: String,
    pub exif_info: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

pub fn bounding_box_overlap(a: BoundingBox, b: BoundingBox) -> f64 {
    let overlap_x = (a.x2.min(b.x2) - a.x1.max(b.x1)).max(0.0);
    let overlap_y = (a.y2.min(b.y2) - a.y1.max(b.y1)).max(0.0);
    let overlap_area = overlap_x * overlap_y;
    let area_a = ((a.x2 - a.x1) * (a.y2 - a.y1)).max(0.0);
    if area_a == 0.0 {
        return 0.0;
    }
    overlap_area / area_a
}

pub fn as_date_string(date: Option<chrono::NaiveDate>) -> Option<String> {
    date.map(|date| format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day()))
}

pub fn as_date_time_string(date: Option<chrono::DateTime<chrono::Utc>>) -> Option<String> {
    date.map(|date| date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

pub fn get_keys_deep(target: &Value) -> Vec<String> {
    fn visit(value: &Value, path: &mut Vec<String>, out: &mut Vec<String>) {
        let Some(object) = value.as_object() else {
            return;
        };
        for (key, child) in object {
            if child.is_null() {
                out.push(join_path(path, key));
                continue;
            }
            if child.is_object() {
                path.push(key.clone());
                visit(child, path, out);
                path.pop();
            } else {
                out.push(join_path(path, key));
            }
        }
    }

    let mut out = Vec::new();
    visit(target, &mut Vec::new(), &mut out);
    out
}

pub fn unset_deep(mut object: Value, key: &str) -> Option<Value> {
    let parts: Vec<&str> = key.split('.').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return Some(object);
    }
    unset_parts(&mut object, &parts);
    (!is_empty_object(&object)).then_some(object)
}

pub fn glob_to_sql_pattern(glob: &str) -> String {
    let mut result = String::with_capacity(glob.len());
    let mut chars = glob.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                }
                result.push('%');
            }
            '?' => result.push('_'),
            _ => result.push(ch),
        }
    }
    result
}

pub fn app_version_from_user_agent(user_agent: &str) -> Option<&str> {
    user_agent
        .strip_prefix("immich-android/")
        .or_else(|| user_agent.strip_prefix("immich-ios/"))
        .or_else(|| user_agent.strip_prefix("immich-unknown/"))
        .or_else(|| user_agent.strip_prefix("Immich_Android_"))
        .or_else(|| user_agent.strip_prefix("Immich_iOS_"))
        .or_else(|| user_agent.strip_prefix("Immich_Unknown_"))
}

pub fn get_exif_count(asset: &DuplicateAsset) -> usize {
    asset
        .exif_info
        .as_ref()
        .and_then(Value::as_object)
        .map(|object| object.values().filter(|value| is_js_truthy(value)).count())
        .unwrap_or_default()
}

pub fn suggest_duplicate(assets: &[DuplicateAsset]) -> Option<&DuplicateAsset> {
    assets.iter().max_by_key(|asset| {
        (
            asset
                .exif_info
                .as_ref()
                .and_then(|exif| exif.get("fileSizeInByte"))
                .and_then(Value::as_i64)
                .unwrap_or_default(),
            get_exif_count(asset),
        )
    })
}

pub fn suggest_duplicate_keep_asset_ids(assets: &[DuplicateAsset]) -> Vec<String> {
    suggest_duplicate(assets)
        .map(|asset| vec![asset.id.clone()])
        .unwrap_or_default()
}

fn join_path(path: &[String], key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{}.{}", path.join("."), key)
    }
}

fn unset_parts(value: &mut Value, parts: &[&str]) -> bool {
    let Value::Object(object) = value else {
        return false;
    };

    if parts.len() == 1 {
        object.remove(parts[0]);
    } else if let Some(child) = object.get_mut(parts[0]) {
        if unset_parts(child, &parts[1..]) {
            object.remove(parts[0]);
        }
    }

    object.is_empty()
}

fn is_empty_object(value: &Value) -> bool {
    matches!(value, Value::Object(object) if object.is_empty())
}

fn is_js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64() != Some(0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

#[allow(dead_code)]
fn empty_object() -> Value {
    Value::Object(Map::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};
    use serde_json::json;

    fn duplicate(id: &str, file_size: Option<i64>, exif: Value) -> DuplicateAsset {
        let exif_info = if file_size.is_none() && exif.as_object().is_none_or(Map::is_empty) {
            None
        } else {
            let mut object = exif.as_object().cloned().unwrap_or_default();
            if let Some(file_size) = file_size {
                object.insert("fileSizeInByte".to_owned(), json!(file_size));
            }
            Some(Value::Object(object))
        };
        DuplicateAsset {
            id: id.to_owned(),
            exif_info,
        }
    }

    fn bbox(x1: f64, y1: f64, x2: f64, y2: f64) -> BoundingBox {
        BoundingBox { x1, y1, x2, y2 }
    }

    #[test]
    fn bounding_box_overlap_matches_immich_editor_cases() {
        let full = bbox(0.0, 0.0, 100.0, 100.0);
        assert_eq!(bounding_box_overlap(full, full), 1.0);
        assert_eq!(
            bounding_box_overlap(full, bbox(200.0, 200.0, 300.0, 300.0)),
            0.0
        );
        assert_eq!(
            bounding_box_overlap(full, bbox(50.0, 0.0, 150.0, 100.0)),
            0.5
        );
        assert_eq!(
            bounding_box_overlap(full, bbox(50.0, 50.0, 150.0, 150.0)),
            0.25
        );
        assert_eq!(
            bounding_box_overlap(bbox(25.0, 25.0, 75.0, 75.0), full),
            1.0
        );
        assert_eq!(
            bounding_box_overlap(full, bbox(25.0, 25.0, 75.0, 75.0)),
            0.25
        );
        assert_eq!(
            bounding_box_overlap(full, bbox(100.0, 0.0, 200.0, 100.0)),
            0.0
        );
        assert_eq!(
            bounding_box_overlap(full, bbox(0.0, 50.0, 100.0, 150.0)),
            0.5
        );
    }

    #[test]
    fn as_date_string_matches_immich_cases() {
        assert_eq!(as_date_string(None), None);
        assert_eq!(
            as_date_string(Some(NaiveDate::from_ymd_opt(2000, 1, 15).unwrap())),
            Some("2000-01-15".to_owned())
        );
        assert_eq!(
            as_date_string(Some(NaiveDate::from_ymd_opt(280, 12, 12).unwrap())),
            Some("0280-12-12".to_owned())
        );
    }

    #[test]
    fn as_date_time_string_matches_immich_cases() {
        assert_eq!(as_date_time_string(None), None);
        assert_eq!(
            as_date_time_string(Some(Utc.with_ymd_and_hms(2000, 1, 15, 12, 0, 0).unwrap())),
            Some("2000-01-15T12:00:00.000Z".to_owned())
        );
    }

    #[test]
    fn get_keys_deep_handles_empty_object() {
        assert_eq!(get_keys_deep(&json!({})), Vec::<String>::new());
    }

    #[test]
    fn get_keys_deep_lists_properties() {
        assert_eq!(
            get_keys_deep(&json!({
                "foo": "bar",
                "flag": true,
                "count": 42,
                "date": "2026-07-14T00:00:00.000Z"
            })),
            ["count", "date", "flag", "foo"]
        );
    }

    #[test]
    fn get_keys_deep_skips_array_indices() {
        assert_eq!(
            get_keys_deep(&json!({"foo": "bar", "hello": ["foo", "bar"]})),
            ["foo", "hello"]
        );
        assert_eq!(
            get_keys_deep(&json!({"foo": "bar", "nested": {"hello": ["foo", "bar"]}})),
            ["foo", "nested.hello"]
        );
    }

    #[test]
    fn get_keys_deep_lists_nested_properties() {
        assert_eq!(
            get_keys_deep(&json!({"foo": "bar", "hello": {"world": true}})),
            ["foo", "hello.world"]
        );
    }

    #[test]
    fn unset_deep_removes_property() {
        assert_eq!(
            unset_deep(json!({"hello": "world", "foo": "bar"}), "foo"),
            Some(json!({"hello": "world"}))
        );
    }

    #[test]
    fn unset_deep_removes_last_property() {
        assert_eq!(unset_deep(json!({"foo": "bar"}), "foo"), None);
    }

    #[test]
    fn unset_deep_removes_nested_property() {
        assert_eq!(
            unset_deep(
                json!({"foo": "bar", "nested": {"enabled": true, "count": 42}}),
                "nested.enabled"
            ),
            Some(json!({"foo": "bar", "nested": {"count": 42}}))
        );
    }

    #[test]
    fn unset_deep_cleans_up_empty_parent() {
        assert_eq!(
            unset_deep(
                json!({"foo": "bar", "nested": {"enabled": true}}),
                "nested.enabled"
            ),
            Some(json!({"foo": "bar"}))
        );
    }

    #[test]
    fn glob_to_sql_pattern_matches_immich_cases() {
        for (input, expected) in [
            ("**/Raw/**", "%/Raw/%"),
            ("**/abc/*.tif", "%/abc/%.tif"),
            ("**/*.tif", "%/%.tif"),
            ("**/*.jp?", "%/%.jp_"),
            ("**/@eaDir/**", "%/@eaDir/%"),
            ("**/._*", "%/._%"),
            ("/absolute/path/**", "/absolute/path/%"),
        ] {
            assert_eq!(glob_to_sql_pattern(input), expected);
        }
    }

    #[test]
    fn app_version_from_user_agent_matches_current_and_legacy_immich_formats() {
        for user_agent in [
            "immich-android/1.123.4",
            "immich-ios/1.123.4",
            "immich-unknown/1.123.4",
            "Immich_Android_1.123.4",
            "Immich_iOS_1.123.4",
            "Immich_Unknown_1.123.4",
        ] {
            assert_eq!(app_version_from_user_agent(user_agent), Some("1.123.4"));
        }
        assert_eq!(app_version_from_user_agent("Mozilla/5.0"), None);
    }

    #[test]
    fn get_exif_count_matches_immich_truthy_count() {
        assert_eq!(get_exif_count(&duplicate("asset-1", None, json!({}))), 0);
        assert_eq!(
            get_exif_count(&duplicate(
                "asset-1",
                Some(1000),
                json!({
                    "make": "Canon",
                    "model": "EOS 5D",
                    "dateTimeOriginal": "2026-07-14T00:00:00.000Z",
                    "timeZone": "UTC",
                    "latitude": 40.7128,
                    "longitude": -74.006,
                    "city": "New York",
                    "state": "NY",
                    "country": "USA",
                    "description": "A photo",
                    "rating": 5
                })
            )),
            12
        );
        assert_eq!(
            get_exif_count(&duplicate(
                "asset-1",
                Some(1000),
                json!({"make": "Canon", "model": null, "latitude": null, "city": "", "rating": 0})
            )),
            2
        );
    }

    #[test]
    fn suggest_duplicate_returns_none_for_empty_list() {
        assert!(suggest_duplicate(&[]).is_none());
        assert_eq!(suggest_duplicate_keep_asset_ids(&[]), Vec::<String>::new());
    }

    #[test]
    fn suggest_duplicate_prefers_largest_file_size() {
        let assets = vec![
            duplicate("small", Some(1000), json!({})),
            duplicate("large", Some(5000), json!({})),
            duplicate("medium", Some(3000), json!({})),
        ];
        assert_eq!(suggest_duplicate(&assets).unwrap().id, "large");
        assert_eq!(suggest_duplicate_keep_asset_ids(&assets), ["large"]);
    }

    #[test]
    fn suggest_duplicate_uses_exif_count_as_tie_breaker() {
        let assets = vec![
            duplicate("less-exif", Some(1000), json!({"make": "Canon"})),
            duplicate(
                "more-exif",
                Some(1000),
                json!({
                    "make": "Canon",
                    "model": "EOS 5D",
                    "dateTimeOriginal": "2026-07-14T00:00:00.000Z",
                    "city": "New York"
                }),
            ),
        ];
        assert_eq!(suggest_duplicate(&assets).unwrap().id, "more-exif");
    }

    #[test]
    fn suggest_duplicate_prioritizes_file_size_over_exif_count() {
        let assets = vec![
            duplicate("large-less-exif", Some(5000), json!({"make": "Canon"})),
            duplicate(
                "small-more-exif",
                Some(1000),
                json!({
                    "make": "Canon",
                    "model": "EOS 5D",
                    "dateTimeOriginal": "2026-07-14T00:00:00.000Z",
                    "city": "New York",
                    "state": "NY",
                    "country": "USA"
                }),
            ),
        ];
        assert_eq!(suggest_duplicate(&assets).unwrap().id, "large-less-exif");
    }
}
