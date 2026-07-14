//! Small protocol helpers ported from Immich's `server/src/utils` tests.

use serde_json::{Map, Value};

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

#[allow(dead_code)]
fn empty_object() -> Value {
    Value::Object(Map::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
