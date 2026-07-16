//! Server info endpoints: /server/about, /server/config, /server/features,
//! /server/version, /server/ping, /server/storage, /server/statistics.
//!
//! Reports the Immich version we are wire-compatible with, so existing
//! clients (immich mobile, CLI) accept the server.

use domus_common::Result;
use domus_db::Repositories;
use serde_json::json;

/// Immich release whose API surface this build implements.
pub const COMPAT_VERSION: (u32, u32, u32) = (3, 0, 3);

pub struct ServerService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl ServerService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub fn version(&self) -> serde_json::Value {
        Self::version_value()
    }

    pub fn version_value() -> serde_json::Value {
        let (major, minor, patch) = COMPAT_VERSION;
        json!({ "major": major, "minor": minor, "patch": patch, "prerelease": null })
    }

    pub fn features(&self) -> serde_json::Value {
        Self::features_value()
    }

    pub fn features_value() -> serde_json::Value {
        json!({
            "smartSearch": true,
            "facialRecognition": true,
            "duplicateDetection": true,
            "ocr": true,
            "importFaces": true,
            "map": true,
            "reverseGeocoding": true,
            "sidecar": true,
            "search": true,
            "trash": true,
            "oauth": false,
            "oauthAutoLaunch": false,
            "passwordLogin": true,
            "configFile": false,
            "email": false,
            "realtimeTranscoding": false,
        })
    }

    pub async fn config(&self) -> Result<serde_json::Value> {
        // TODO: read system_metadata (admin onboarding, external domain...).
        Ok(json!({
            "loginPageMessage": "",
            "trashDays": 30,
            "userDeleteDelay": 7,
            "oauthButtonText": "Login with OAuth",
            "isInitialized": true,
            "isOnboarded": false,
            "externalDomain": "",
            "publicUsers": true,
            "mapDarkStyleUrl": "https://tiles.immich.cloud/v1/style/dark.json",
            "mapLightStyleUrl": "https://tiles.immich.cloud/v1/style/light.json",
            "maintenanceMode": false,
            "minFaces": 3,
        }))
    }

    pub async fn about(&self) -> Result<serde_json::Value> {
        let (major, minor, patch) = COMPAT_VERSION;
        Ok(json!({
            "version": format!("v{major}.{minor}.{patch}"),
            "versionUrl": "https://github.com/xiedeacc/domus",
            "sourceRef": "domus",
            "build": "domus",
            "licensed": false,
        }))
    }

    pub async fn storage(&self) -> Result<serde_json::Value> {
        // TODO: statvfs on the media location.
        Ok(json!({
            "diskSize": "0 B", "diskUse": "0 B", "diskAvailable": "0 B",
            "diskSizeRaw": 0, "diskUseRaw": 0, "diskAvailableRaw": 0,
            "diskUsagePercentage": 0.0,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_immich_version_shape() {
        assert_eq!(
            ServerService::version_value(),
            json!({ "major": 3, "minor": 0, "patch": 3, "prerelease": null })
        );
    }

    #[test]
    fn features_enable_rust_ml_related_capabilities() {
        let features = ServerService::features_value();
        assert_eq!(features["smartSearch"], true);
        assert_eq!(features["duplicateDetection"], true);
        assert_eq!(features["facialRecognition"], true);
        assert_eq!(features["ocr"], true);
        assert_eq!(features["passwordLogin"], true);
        assert_eq!(features["map"], true);
    }
}
