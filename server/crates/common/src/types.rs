//! Core enums shared across layers. Serialized names must match the Immich
//! OpenAPI spec exactly — they are part of the wire protocol.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetType {
    Image,
    Video,
    Audio,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetVisibility {
    Archive,
    Timeline,
    Hidden,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UserStatus {
    Active,
    Removing,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserAvatarColor {
    Primary,
    Pink,
    Red,
    Yellow,
    Blue,
    Green,
    Purple,
    Orange,
    Gray,
    Amber,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlbumUserRole {
    Editor,
    Viewer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharedLinkType {
    #[serde(rename = "ALBUM", alias = "album")]
    Album,
    #[serde(rename = "INDIVIDUAL", alias = "individual")]
    Individual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[cfg(test)]
mod tests {
    use super::SharedLinkType;

    #[test]
    fn shared_link_type_serializes_like_immich_enum() {
        assert_eq!(
            serde_json::to_string(&SharedLinkType::Individual).unwrap(),
            "\"INDIVIDUAL\""
        );
        assert_eq!(
            serde_json::to_string(&SharedLinkType::Album).unwrap(),
            "\"ALBUM\""
        );
    }

    #[test]
    fn shared_link_type_accepts_legacy_lowercase_inputs() {
        assert_eq!(
            serde_json::from_str::<SharedLinkType>("\"individual\"").unwrap(),
            SharedLinkType::Individual
        );
        assert_eq!(
            serde_json::from_str::<SharedLinkType>("\"album\"").unwrap(),
            SharedLinkType::Album
        );
    }
}
