use domus_common::{Error, Result};
use domus_db::Repositories;
use serde_json::Value;

const SYSTEM_CONFIG_KEY: &str = "system-config";

pub struct SystemConfigService {
    repos: Repositories,
}

impl SystemConfigService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn get(&self) -> Result<serde_json::Value> {
        Ok(self
            .repos
            .system_metadata
            .get(SYSTEM_CONFIG_KEY)
            .await?
            .unwrap_or_else(default_config))
    }

    pub async fn set(&self, value: serde_json::Value) -> Result<serde_json::Value> {
        let value = merge_with_defaults(value);
        validate_config(&value)?;
        self.repos
            .system_metadata
            .set(SYSTEM_CONFIG_KEY, value.clone())
            .await?;
        Ok(value)
    }

    pub async fn storage_template(&self) -> Result<serde_json::Value> {
        Ok(self
            .get()
            .await?
            .get("storageTemplate")
            .cloned()
            .unwrap_or_else(default_storage_template))
    }
}

pub fn default_config() -> serde_json::Value {
    serde_json::json!({
        "backup": {
            "database": {
                "enabled": true,
                "cronExpression": "0 02 * * *",
                "keepLastAmount": 14
            }
        },
        "ffmpeg": {
            "crf": 23,
            "threads": 0,
            "preset": "ultrafast",
            "targetVideoCodec": "h264",
            "acceptedVideoCodecs": ["h264"],
            "targetAudioCodec": "aac",
            "acceptedAudioCodecs": ["aac", "mp3", "opus"],
            "acceptedContainers": ["mov", "ogg", "webm"],
            "targetResolution": "720",
            "maxBitrate": "0",
            "bframes": -1,
            "refs": 0,
            "gopSize": 0,
            "temporalAQ": false,
            "cqMode": "auto",
            "twoPass": false,
            "preferredHwDevice": "auto",
            "transcode": "required",
            "tonemap": "hable",
            "accel": "disabled",
            "accelDecode": true,
            "realtime": {
                "enabled": false,
                "videoCodecs": ["h264", "hevc"],
                "resolutions": [480, 720, 1080]
            }
        },
        "integrityChecks": {
            "missingFiles": {"enabled": true, "cronExpression": "0 03 * * *"},
            "untrackedFiles": {"enabled": true, "cronExpression": "0 03 * * *"},
            "checksumFiles": {
                "enabled": true,
                "cronExpression": "0 03 * * *",
                "timeLimit": 3600000,
                "percentageLimit": 1
            }
        },
        "job": {
            "backgroundTask": {"concurrency": 5},
            "smartSearch": {"concurrency": 2},
            "metadataExtraction": {"concurrency": 5},
            "faceDetection": {"concurrency": 2},
            "search": {"concurrency": 5},
            "sidecar": {"concurrency": 5},
            "library": {"concurrency": 5},
            "migration": {"concurrency": 5},
            "thumbnailGeneration": {"concurrency": 3},
            "videoConversion": {"concurrency": 1},
            "notifications": {"concurrency": 5},
            "ocr": {"concurrency": 1},
            "workflow": {"concurrency": 5},
            "integrityCheck": {"concurrency": 1},
            "editor": {"concurrency": 2}
        },
        "logging": {
            "enabled": true,
            "level": "log"
        },
        "machineLearning": {
            "enabled": false,
            "urls": ["http://immich-machine-learning:3003"],
            "availabilityChecks": {"enabled": false, "timeout": 2000, "interval": 30000},
            "clip": {"enabled": false, "modelName": "ViT-B-32__openai"},
            "duplicateDetection": {"enabled": false, "maxDistance": 0.01},
            "facialRecognition": {
                "enabled": false,
                "modelName": "buffalo_l",
                "minScore": 0.7,
                "maxDistance": 0.5,
                "minFaces": 3
            },
            "ocr": {
                "enabled": false,
                "modelName": "PP-OCRv5_mobile",
                "minDetectionScore": 0.5,
                "minRecognitionScore": 0.8,
                "maxResolution": 736
            }
        },
        "map": {
            "enabled": true,
            "lightStyle": "https://tiles.immich.cloud/v1/style/light.json",
            "darkStyle": "https://tiles.immich.cloud/v1/style/dark.json"
        },
        "reverseGeocoding": {
            "enabled": true
        },
        "metadata": {
            "faces": {"import": false}
        },
        "oauth": {
            "enabled": false,
            "autoLaunch": false,
            "autoRegister": true,
            "buttonText": "Login with OAuth",
            "clientId": "",
            "clientSecret": "",
            "defaultStorageQuota": null,
            "issuerUrl": "",
            "endSessionEndpoint": "",
            "mobileOverrideEnabled": false,
            "mobileRedirectUri": "",
            "prompt": "",
            "scope": "openid email profile",
            "signingAlgorithm": "RS256",
            "profileSigningAlgorithm": "none",
            "storageLabelClaim": "preferred_username",
            "storageQuotaClaim": "immich_quota",
            "roleClaim": "immich_role",
            "tokenEndpointAuthMethod": "client_secret_post",
            "timeout": 30000,
            "allowInsecureRequests": false,
            "authorizeUrl": null,
            "tokenEndpoint": null,
            "userinfoEndpoint": null
        },
        "passwordLogin": {
            "enabled": true
        },
        "storageTemplate": default_storage_template(),
        "image": {
            "thumbnail": {
                "format": "webp",
                "size": 512,
                "quality": 95,
                "progressive": false
            },
            "preview": {
                "format": "jpeg",
                "size": 2560,
                "quality": 95,
                "progressive": false
            },
            "colorspace": "p3",
            "extractEmbedded": false,
            "fullsize": {
                "enabled": false,
                "format": "jpeg",
                "quality": 95,
                "progressive": false
            }
        },
        "newVersionCheck": {
            "enabled": true,
            "channel": "stable"
        },
        "nightlyTasks": {
            "startTime": "00:00",
            "databaseCleanup": true,
            "generateMemories": true,
            "syncQuotaUsage": true,
            "missingThumbnails": true,
            "clusterNewFaces": true
        },
        "trash": {
            "enabled": true,
            "days": 30
        },
        "theme": {
            "customCss": ""
        },
        "library": {
            "scan": {"enabled": true, "cronExpression": "0 0 * * *"},
            "watch": {"enabled": false}
        },
        "server": {
            "externalDomain": "",
            "loginPageMessage": "",
            "publicUsers": true
        },
        "notifications": {
            "smtp": {
                "enabled": false,
                "from": "",
                "replyTo": "",
                "transport": {
                    "ignoreCert": false,
                    "host": "",
                    "port": 587,
                    "secure": false,
                    "username": "",
                    "password": ""
                }
            }
        },
        "templates": {
            "email": {
                "welcomeTemplate": "",
                "albumInviteTemplate": "",
                "albumUpdateTemplate": ""
            }
        },
        "user": {
            "deleteDelay": 7
        }
    })
}

pub fn default_storage_template() -> serde_json::Value {
    serde_json::json!({
        "enabled": false,
        "hashVerificationEnabled": true,
        "template": "{{y}}/{{y}}-{{MM}}-{{dd}}/{{filename}}",
    })
}

pub fn merge_with_defaults(value: Value) -> Value {
    let defaults = default_config();
    let mut merged = merge_json(defaults.clone(), value);
    coerce_bool_strings(&defaults, &mut merged);
    merged
}

pub fn validate_config(value: &Value) -> Result<()> {
    validate_against_schema(&default_config(), value, &[])?;
    validate_nightly_tasks(value)?;
    validate_constraints(value)?;
    Ok(())
}

fn merge_json(mut base: Value, overlay: Value) -> Value {
    match (&mut base, overlay) {
        (Value::Object(base), Value::Object(overlay)) => {
            for (key, value) in overlay {
                let merged = match base.remove(&key) {
                    Some(existing) => merge_json(existing, value),
                    None => value,
                };
                base.insert(key, merged);
            }
            Value::Object(std::mem::take(base))
        }
        (_, overlay) => overlay,
    }
}

fn coerce_bool_strings(schema: &Value, value: &mut Value) {
    match schema {
        Value::Bool(_) => {
            if let Value::String(text) = value {
                if text == "true" || text == "false" {
                    *value = Value::Bool(text == "true");
                }
            }
        }
        Value::Object(schema) => {
            if let Value::Object(value) = value {
                for (key, child) in value {
                    if let Some(child_schema) = schema.get(key) {
                        coerce_bool_strings(child_schema, child);
                    }
                }
            }
        }
        Value::Array(schema) => {
            if let (Some(item_schema), Value::Array(value)) = (schema.first(), value) {
                for item in value {
                    coerce_bool_strings(item_schema, item);
                }
            }
        }
        _ => {}
    }
}

fn validate_against_schema(schema: &Value, value: &Value, path: &[&str]) -> Result<()> {
    match (schema, value) {
        (Value::Object(schema), Value::Object(value)) => {
            for (key, child) in value {
                if let Some(child_schema) = schema.get(key) {
                    let mut child_path = path.to_vec();
                    child_path.push(key);
                    validate_against_schema(child_schema, child, &child_path)?;
                }
            }
            Ok(())
        }
        (Value::Array(schema), Value::Array(value)) => {
            if let Some(item_schema) = schema.first() {
                for item in value {
                    validate_against_schema(item_schema, item, path)?;
                }
            }
            Ok(())
        }
        (Value::Bool(_), Value::Bool(_))
        | (Value::String(_), Value::String(_))
        | (Value::Number(_), Value::Number(_))
        | (Value::Null, _) => Ok(()),
        _ => Err(validation_error(path, schema, value)),
    }
}

fn validate_nightly_tasks(value: &Value) -> Result<()> {
    let Some(tasks) = value.get("nightlyTasks").and_then(Value::as_object) else {
        return Ok(());
    };
    if let Some(start_time) = tasks.get("startTime") {
        let Some(start_time) = start_time.as_str() else {
            return Err(Error::BadRequest(
                "Validation failed: nightlyTasks.startTime must be a string in HH:MM format".into(),
            ));
        };
        if !is_hh_mm(start_time) {
            return Err(Error::BadRequest(
                "Validation failed: nightlyTasks.startTime must be a string in HH:MM format".into(),
            ));
        }
    }
    Ok(())
}

fn validate_constraints(value: &Value) -> Result<()> {
    validate_int_range(value, "/backup/database/keepLastAmount", Some(1), None)?;
    validate_cron(value, "/backup/database/cronExpression")?;
    validate_int_range(value, "/ffmpeg/crf", Some(0), Some(51))?;
    validate_int_range(value, "/ffmpeg/threads", Some(0), None)?;
    validate_int_range(value, "/ffmpeg/bframes", Some(-1), Some(16))?;
    validate_int_range(value, "/ffmpeg/refs", Some(0), Some(6))?;
    validate_int_range(value, "/ffmpeg/gopSize", Some(0), None)?;
    validate_enum(
        value,
        "/ffmpeg/targetVideoCodec",
        &["h264", "hevc", "vp9", "av1"],
    )?;
    validate_enum_array(
        value,
        "/ffmpeg/acceptedVideoCodecs",
        &["h264", "hevc", "vp9", "av1"],
    )?;
    validate_enum(
        value,
        "/ffmpeg/targetAudioCodec",
        &["mp3", "aac", "opus", "pcm_s16le"],
    )?;
    validate_enum_array(
        value,
        "/ffmpeg/acceptedAudioCodecs",
        &["mp3", "aac", "opus", "pcm_s16le"],
    )?;
    validate_enum_array(
        value,
        "/ffmpeg/acceptedContainers",
        &["mov", "mp4", "ogg", "webm"],
    )?;
    validate_enum(value, "/ffmpeg/cqMode", &["auto", "cqp", "icq"])?;
    validate_enum(
        value,
        "/ffmpeg/transcode",
        &["all", "optimal", "bitrate", "required", "disabled"],
    )?;
    validate_enum(
        value,
        "/ffmpeg/accel",
        &["nvenc", "qsv", "vaapi", "rkmpp", "disabled"],
    )?;
    validate_enum(
        value,
        "/ffmpeg/tonemap",
        &["hable", "mobius", "reinhard", "disabled"],
    )?;
    validate_enum_array(
        value,
        "/ffmpeg/realtime/videoCodecs",
        &["h264", "hevc", "vp9", "av1"],
    )?;
    validate_i64_array(
        value,
        "/ffmpeg/realtime/resolutions",
        &[480, 720, 1080, 1440, 2160],
    )?;

    validate_cron(value, "/integrityChecks/missingFiles/cronExpression")?;
    validate_cron(value, "/integrityChecks/untrackedFiles/cronExpression")?;
    validate_cron(value, "/integrityChecks/checksumFiles/cronExpression")?;
    validate_int_range(
        value,
        "/integrityChecks/checksumFiles/timeLimit",
        Some(0),
        None,
    )?;
    validate_number_range(
        value,
        "/integrityChecks/checksumFiles/percentageLimit",
        Some(0.0),
        Some(1.0),
    )?;

    if let Some(jobs) = value.get("job").and_then(Value::as_object) {
        for key in jobs.keys() {
            validate_int_range(value, &format!("/job/{key}/concurrency"), Some(1), None)?;
        }
    }
    validate_cron(value, "/library/scan/cronExpression")?;
    validate_enum(
        value,
        "/logging/level",
        &["verbose", "debug", "log", "warn", "error", "fatal"],
    )?;
    validate_non_empty_array(value, "/machineLearning/urls")?;
    validate_url(value, "/map/lightStyle", false)?;
    validate_url(value, "/map/darkStyle", false)?;
    validate_enum(
        value,
        "/newVersionCheck/channel",
        &["stable", "releaseCandidate"],
    )?;
    validate_int_range(value, "/oauth/timeout", Some(1), None)?;
    validate_nullable_int_range(value, "/oauth/defaultStorageQuota", Some(0), None)?;
    validate_enum(
        value,
        "/oauth/tokenEndpointAuthMethod",
        &["client_secret_post", "client_secret_basic"],
    )?;
    validate_url(value, "/oauth/issuerUrl", true)?;
    validate_url(value, "/oauth/endSessionEndpoint", true)?;
    if value
        .pointer("/oauth/mobileOverrideEnabled")
        .and_then(Value::as_bool)
        == Some(true)
    {
        validate_url(value, "/oauth/mobileRedirectUri", true)?;
    }
    validate_url(value, "/server/externalDomain", true)?;
    validate_int_range(
        value,
        "/notifications/smtp/transport/port",
        Some(0),
        Some(65_535),
    )?;
    validate_enum(value, "/image/thumbnail/format", &["jpeg", "webp"])?;
    validate_int_range(value, "/image/thumbnail/quality", Some(1), Some(100))?;
    validate_int_range(value, "/image/thumbnail/size", Some(1), None)?;
    validate_enum(value, "/image/preview/format", &["jpeg", "webp"])?;
    validate_int_range(value, "/image/preview/quality", Some(1), Some(100))?;
    validate_int_range(value, "/image/preview/size", Some(1), None)?;
    validate_enum(value, "/image/fullsize/format", &["jpeg", "webp"])?;
    validate_int_range(value, "/image/fullsize/quality", Some(1), Some(100))?;
    validate_enum(value, "/image/colorspace", &["srgb", "p3"])?;
    validate_int_range(value, "/trash/days", Some(0), None)?;
    validate_int_range(value, "/user/deleteDelay", Some(1), None)?;
    Ok(())
}

fn validate_cron(value: &Value, path: &str) -> Result<()> {
    let Some(cron) = value.pointer(path).and_then(Value::as_str) else {
        return Ok(());
    };
    let parts = cron.split_whitespace().count();
    if parts == 5 || parts == 6 {
        return Ok(());
    }
    Err(path_error(path, "a valid cron expression"))
}

fn validate_url(value: &Value, path: &str, allow_empty: bool) -> Result<()> {
    let Some(url) = value.pointer(path).and_then(Value::as_str) else {
        return Ok(());
    };
    if (allow_empty && url.is_empty()) || looks_like_url(url) {
        return Ok(());
    }
    Err(path_error(path, "an empty string or valid URL"))
}

fn validate_enum(value: &Value, path: &str, allowed: &[&str]) -> Result<()> {
    let Some(actual) = value.pointer(path).and_then(Value::as_str) else {
        return Ok(());
    };
    if allowed.contains(&actual) {
        return Ok(());
    }
    Err(path_error(path, &format!("one of {}", allowed.join(", "))))
}

fn validate_enum_array(value: &Value, path: &str, allowed: &[&str]) -> Result<()> {
    let Some(items) = value.pointer(path).and_then(Value::as_array) else {
        return Ok(());
    };
    for item in items {
        let Some(actual) = item.as_str() else {
            return Err(path_error(path, "array of strings"));
        };
        if !allowed.contains(&actual) {
            return Err(path_error(
                path,
                &format!("array values from {}", allowed.join(", ")),
            ));
        }
    }
    Ok(())
}

fn validate_i64_array(value: &Value, path: &str, allowed: &[i64]) -> Result<()> {
    let Some(items) = value.pointer(path).and_then(Value::as_array) else {
        return Ok(());
    };
    for item in items {
        let Some(actual) = as_i64(item) else {
            return Err(path_error(path, "array of integers"));
        };
        if !allowed.contains(&actual) {
            return Err(path_error(
                path,
                "array values from 480, 720, 1080, 1440, 2160",
            ));
        }
    }
    Ok(())
}

fn validate_non_empty_array(value: &Value, path: &str) -> Result<()> {
    let Some(items) = value.pointer(path).and_then(Value::as_array) else {
        return Ok(());
    };
    if items.is_empty() {
        return Err(path_error(path, "a non-empty array"));
    }
    Ok(())
}

fn validate_nullable_int_range(
    value: &Value,
    path: &str,
    min: Option<i64>,
    max: Option<i64>,
) -> Result<()> {
    if value.pointer(path).is_some_and(Value::is_null) {
        return Ok(());
    }
    validate_int_range(value, path, min, max)
}

fn validate_int_range(value: &Value, path: &str, min: Option<i64>, max: Option<i64>) -> Result<()> {
    let Some(actual) = value.pointer(path) else {
        return Ok(());
    };
    let Some(actual) = as_i64(actual) else {
        return Err(path_error(path, "integer"));
    };
    if min.is_some_and(|min| actual < min) || max.is_some_and(|max| actual > max) {
        return Err(path_error(path, &range_label(min, max)));
    }
    Ok(())
}

fn validate_number_range(
    value: &Value,
    path: &str,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<()> {
    let Some(actual) = value.pointer(path).and_then(Value::as_f64) else {
        return Ok(());
    };
    if min.is_some_and(|min| actual < min) || max.is_some_and(|max| actual > max) {
        return Err(path_error(path, &range_label_f64(min, max)));
    }
    Ok(())
}

fn looks_like_url(value: &str) -> bool {
    let Some(rest) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };
    !rest.is_empty() && !rest.chars().any(char::is_whitespace)
}

fn as_i64(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
}

fn range_label(min: Option<i64>, max: Option<i64>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("integer between {min} and {max}"),
        (Some(min), None) => format!("integer greater than or equal to {min}"),
        (None, Some(max)) => format!("integer less than or equal to {max}"),
        (None, None) => "integer".into(),
    }
}

fn range_label_f64(min: Option<f64>, max: Option<f64>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("number between {min} and {max}"),
        (Some(min), None) => format!("number greater than or equal to {min}"),
        (None, Some(max)) => format!("number less than or equal to {max}"),
        (None, None) => "number".into(),
    }
}

fn path_error(path: &str, expected: &str) -> Error {
    Error::BadRequest(format!(
        "Validation failed: {} expected {}",
        path.trim_start_matches('/').replace('/', "."),
        expected
    ))
}

fn is_hh_mm(value: &str) -> bool {
    let Some((hour, minute)) = value.split_once(':') else {
        return false;
    };
    if hour.len() != 2 || minute.len() != 2 {
        return false;
    }
    let Ok(hour) = hour.parse::<u8>() else {
        return false;
    };
    let Ok(minute) = minute.parse::<u8>() else {
        return false;
    };
    hour < 24 && minute < 60
}

fn validation_error(path: &[&str], schema: &Value, value: &Value) -> Error {
    Error::BadRequest(format!(
        "Validation failed: {} expected {}, received {}",
        path.join("."),
        json_kind(schema),
        json_kind(value)
    ))
}

fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "any",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_contains_immich_oauth_shape() {
        let config = default_config();
        let oauth = config.get("oauth").unwrap();
        assert_eq!(oauth.get("enabled").and_then(|v| v.as_bool()), Some(false));
        assert_eq!(
            oauth.get("autoRegister").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            oauth.get("buttonText").and_then(|v| v.as_str()),
            Some("Login with OAuth")
        );
        assert_eq!(
            oauth.get("scope").and_then(|v| v.as_str()),
            Some("openid email profile")
        );
        for field in [
            "authorizeUrl",
            "tokenEndpoint",
            "userinfoEndpoint",
            "clientId",
            "clientSecret",
        ] {
            assert!(oauth.get(field).is_some(), "{field} missing");
        }
    }

    #[test]
    fn default_config_contains_storage_template_shape() {
        let config = default_config();
        let storage = config.get("storageTemplate").unwrap();
        assert_eq!(
            storage.get("enabled").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            storage.get("template").and_then(|v| v.as_str()),
            Some("{{y}}/{{y}}-{{MM}}-{{dd}}/{{filename}}")
        );
        assert_eq!(
            storage
                .get("hashVerificationEnabled")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn default_config_contains_non_ml_immich_sections() {
        let config = default_config();
        for section in [
            "backup",
            "ffmpeg",
            "integrityChecks",
            "job",
            "logging",
            "map",
            "nightlyTasks",
            "image",
            "server",
            "notifications",
            "user",
        ] {
            assert!(config.get(section).is_some(), "{section} missing");
        }
        assert_eq!(
            config
                .pointer("/machineLearning/enabled")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            config
                .pointer("/job/notifications/concurrency")
                .and_then(|v| v.as_u64()),
            Some(5)
        );
        assert_eq!(
            config.pointer("/ffmpeg/realtime/resolutions"),
            Some(&serde_json::json!([480, 720, 1080]))
        );
    }

    #[test]
    fn default_image_config_prefers_lan_quality() {
        let config = default_config();
        assert_eq!(
            config
                .pointer("/image/thumbnail/size")
                .and_then(|v| v.as_u64()),
            Some(512)
        );
        assert_eq!(
            config
                .pointer("/image/preview/size")
                .and_then(|v| v.as_u64()),
            Some(2560)
        );
        assert_eq!(
            config
                .pointer("/image/preview/quality")
                .and_then(|v| v.as_u64()),
            Some(95)
        );
    }

    #[test]
    fn merges_partial_config_with_defaults() {
        let config = merge_with_defaults(serde_json::json!({
            "oauth": {"autoLaunch": true},
            "trash": {"days": 10}
        }));
        assert_eq!(
            config
                .pointer("/oauth/autoLaunch")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            config.pointer("/oauth/buttonText").and_then(|v| v.as_str()),
            Some("Login with OAuth")
        );
        assert_eq!(
            config.pointer("/trash/days").and_then(|v| v.as_u64()),
            Some(10)
        );
    }

    #[test]
    fn validates_nightly_tasks_start_time_like_immich() {
        let mut config = default_config();
        config["nightlyTasks"]["startTime"] = serde_json::json!("05:05");
        validate_config(&config).unwrap();

        config["nightlyTasks"]["startTime"] = serde_json::json!("invalid");
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("nightlyTasks.startTime"));
        assert!(err.contains("HH:MM"));
    }

    #[test]
    fn validates_nightly_tasks_boolean_fields_like_immich() {
        let mut config = default_config();
        config["nightlyTasks"]["databaseCleanup"] = serde_json::json!("invalid");
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("nightlyTasks.databaseCleanup"));
        assert!(err.contains("expected boolean"));
    }

    #[test]
    fn coerces_config_bool_strings_like_immich() {
        let config = merge_with_defaults(serde_json::json!({
            "backup": {"database": {"enabled": "false"}},
            "ffmpeg": {"twoPass": "true"},
            "oauth": {"enabled": "true"},
            "nightlyTasks": {"databaseCleanup": "false"}
        }));
        validate_config(&config).unwrap();
        assert_eq!(
            config
                .pointer("/backup/database/enabled")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            config.pointer("/ffmpeg/twoPass").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            config.pointer("/oauth/enabled").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            config
                .pointer("/nightlyTasks/databaseCleanup")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn accepts_missing_or_true_image_progressive_like_immich() {
        let mut config = default_config();
        config["image"]["thumbnail"]
            .as_object_mut()
            .unwrap()
            .remove("progressive");
        config["image"]["preview"]
            .as_object_mut()
            .unwrap()
            .remove("progressive");
        config["image"]["fullsize"]
            .as_object_mut()
            .unwrap()
            .remove("progressive");
        let merged = merge_with_defaults(config);
        validate_config(&merged).unwrap();

        let mut config = default_config();
        config["image"]["thumbnail"]["progressive"] = serde_json::json!(true);
        config["image"]["preview"]["progressive"] = serde_json::json!(true);
        config["image"]["fullsize"]["progressive"] = serde_json::json!(true);
        validate_config(&config).unwrap();
    }

    #[test]
    fn rejects_invalid_image_progressive_like_immich() {
        let mut config = default_config();
        config["image"]["thumbnail"]["progressive"] = serde_json::json!("invalid");
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("image.thumbnail.progressive"));
        assert!(err.contains("expected boolean"));
    }

    #[test]
    fn validates_numeric_ranges_from_immich_system_config_schema() {
        let mut config = default_config();
        config["ffmpeg"]["crf"] = serde_json::json!(52);
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("ffmpeg.crf"));
        assert!(err.contains("between 0 and 51"));

        let mut config = default_config();
        config["job"]["thumbnailGeneration"]["concurrency"] = serde_json::json!(0);
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("job.thumbnailGeneration.concurrency"));
        assert!(err.contains("greater than or equal to 1"));

        let mut config = default_config();
        config["notifications"]["smtp"]["transport"]["port"] = serde_json::json!(65_536);
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("notifications.smtp.transport.port"));
        assert!(err.contains("between 0 and 65535"));
    }

    #[test]
    fn validates_image_ranges_and_enums_from_immich_system_config_schema() {
        let mut config = default_config();
        config["image"]["preview"]["quality"] = serde_json::json!(101);
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("image.preview.quality"));
        assert!(err.contains("between 1 and 100"));

        let mut config = default_config();
        config["image"]["thumbnail"]["format"] = serde_json::json!("png");
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("image.thumbnail.format"));
        assert!(err.contains("jpeg, webp"));

        let mut config = default_config();
        config["image"]["colorspace"] = serde_json::json!("adobe-rgb");
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("image.colorspace"));
        assert!(err.contains("srgb, p3"));
    }

    #[test]
    fn validates_url_and_cron_constraints_from_immich_system_config_schema() {
        let mut config = default_config();
        config["map"]["lightStyle"] = serde_json::json!("not a url");
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("map.lightStyle"));
        assert!(err.contains("valid URL"));

        let mut config = default_config();
        config["server"]["externalDomain"] = serde_json::json!("https://photos.example.test");
        config["oauth"]["issuerUrl"] = serde_json::json!("");
        config["oauth"]["mobileOverrideEnabled"] = serde_json::json!(true);
        config["oauth"]["mobileRedirectUri"] = serde_json::json!("domus://callback");
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("oauth.mobileRedirectUri"));

        let mut config = default_config();
        config["library"]["scan"]["cronExpression"] = serde_json::json!("invalid");
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("library.scan.cronExpression"));
        assert!(err.contains("cron"));
    }

    #[test]
    fn validates_ffmpeg_enum_arrays_from_immich_system_config_schema() {
        let mut config = default_config();
        config["ffmpeg"]["acceptedVideoCodecs"] = serde_json::json!(["h264", "theora"]);
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("ffmpeg.acceptedVideoCodecs"));
        assert!(err.contains("h264"));

        let mut config = default_config();
        config["ffmpeg"]["realtime"]["resolutions"] = serde_json::json!([360]);
        let err = validate_config(&config).unwrap_err().to_string();
        assert!(err.contains("ffmpeg.realtime.resolutions"));
        assert!(err.contains("480"));
    }
}
