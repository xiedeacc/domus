use domus_common::Result;
use domus_db::Repositories;

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
        "oauth": {
            "enabled": false,
            "autoLaunch": false,
            "buttonText": "Login with OAuth",
            "authorizeUrl": null,
            "tokenEndpoint": null,
            "userinfoEndpoint": null,
            "clientId": null,
            "clientSecret": null,
            "scope": "openid email profile"
        },
        "storageTemplate": default_storage_template(),
    })
}

pub fn default_storage_template() -> serde_json::Value {
    serde_json::json!({
        "enabled": false,
        "template": "{{y}}/{{MM}}/{{filename}}",
    })
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
            Some("{{y}}/{{MM}}/{{filename}}")
        );
    }
}
