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
                "resolutions": ["480", "720", "1080"]
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
            "notification": {"concurrency": 5},
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
    merge_json(default_config(), value)
}

pub fn validate_config(value: &Value) -> Result<()> {
    validate_against_schema(&default_config(), value, &[])?;
    validate_nightly_tasks(value)?;
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
}
