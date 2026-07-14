//! Public share links (key or slug in query string).

use base64::Engine;
use chrono::{DateTime, Utc};
use domus_common::types::SharedLinkType;
use domus_common::{Error, Result};
use domus_db::entities::SharedLink;
use domus_db::Repositories;
use rand::RngCore;
use uuid::Uuid;

pub struct SharedLinkService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl SharedLinkService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn resolve(
        &self,
        key: Option<&str>,
        slug: Option<&str>,
    ) -> Result<domus_db::entities::SharedLink> {
        match (key, slug) {
            (Some(k), _) => {
                let bytes = decode_shared_link_key(k)?;
                self.repos
                    .shared_link
                    .get_by_key(&bytes)
                    .await?
                    .ok_or_else(|| Error::Unauthorized("invalid share key".into()))
            }
            (None, Some(s)) => self
                .repos
                .shared_link
                .get_by_slug(s)
                .await?
                .ok_or_else(|| Error::Unauthorized("invalid share slug".into())),
            _ => Err(Error::BadRequest("missing key or slug".into())),
        }
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<domus_db::entities::SharedLink>> {
        self.repos.shared_link.list_for_user(user_id).await
    }

    pub async fn get(&self, id: Uuid) -> Result<SharedLink> {
        self.repos.shared_link.get(id).await
    }

    pub async fn assets(&self, id: Uuid) -> Result<Vec<domus_db::entities::Asset>> {
        let link = self.repos.shared_link.get(id).await?;
        let asset_ids = self.repos.shared_link.asset_ids(id).await?;
        self.repos.asset.list_by_ids(link.user_id, &asset_ids).await
    }

    pub async fn asset_ids(&self, id: Uuid) -> Result<Vec<Uuid>> {
        self.repos.shared_link.asset_ids(id).await
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        link_type: SharedLinkType,
        album_id: Option<Uuid>,
        asset_ids: &[Uuid],
        description: Option<String>,
        password: Option<String>,
        slug: Option<String>,
        allow_upload: bool,
        allow_download: bool,
        show_metadata: bool,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<SharedLink> {
        validate_create_request(link_type, album_id, asset_ids)?;

        let mut key = vec![0; 32];
        rand::rng().fill_bytes(&mut key);
        let link = SharedLink {
            id: Uuid::new_v4(),
            user_id,
            key,
            slug,
            link_type,
            album_id,
            description,
            password,
            allow_upload,
            allow_download: effective_allow_download(allow_download, show_metadata),
            show_exif: show_metadata,
            expires_at,
            created_at: Utc::now(),
        };
        self.repos.shared_link.create(link, asset_ids).await
    }

    pub async fn update_options(
        &self,
        id: Uuid,
        allow_upload: Option<bool>,
        allow_download: Option<bool>,
        show_metadata: Option<bool>,
        description: Option<&str>,
        password: Option<&str>,
        slug: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<SharedLink> {
        let allow_download = match (allow_download, show_metadata) {
            (_, Some(false)) => Some(false),
            (value, _) => value,
        };
        self.repos
            .shared_link
            .update_options(
                id,
                allow_upload,
                allow_download,
                show_metadata,
                description,
                password,
                slug,
                expires_at,
            )
            .await
    }

    pub async fn add_assets(&self, id: Uuid, asset_ids: &[Uuid]) -> Result<()> {
        self.repos.shared_link.add_assets(id, asset_ids).await
    }

    pub async fn remove_assets(&self, id: Uuid, asset_ids: &[Uuid]) -> Result<()> {
        self.repos.shared_link.remove_assets(id, asset_ids).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        self.repos.shared_link.delete(id).await
    }
}

fn validate_create_request(
    link_type: SharedLinkType,
    album_id: Option<Uuid>,
    asset_ids: &[Uuid],
) -> Result<()> {
    match link_type {
        SharedLinkType::Album if album_id.is_none() => Err(Error::BadRequest(
            "album shared link requires albumId".into(),
        )),
        SharedLinkType::Individual if asset_ids.is_empty() => Err(Error::BadRequest(
            "individual shared link requires assetIds".into(),
        )),
        _ => Ok(()),
    }
}

fn effective_allow_download(allow_download: bool, show_metadata: bool) -> bool {
    allow_download && show_metadata
}

pub(crate) fn decode_shared_link_key(value: &str) -> Result<Vec<u8>> {
    if let Some(decoded) = decode_32(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(value))
    {
        return Ok(decoded);
    }
    if let Some(decoded) = decode_32(base64::engine::general_purpose::URL_SAFE.decode(value)) {
        return Ok(decoded);
    }
    if let Some(decoded) = decode_32(hex::decode(value)) {
        return Ok(decoded);
    }
    Err(Error::BadRequest("invalid key".into()))
}

fn decode_32<E>(decoded: std::result::Result<Vec<u8>, E>) -> Option<Vec<u8>> {
    decoded.ok().filter(|value| value.len() == 32)
}

#[cfg(test)]
mod tests {
    use super::{decode_shared_link_key, effective_allow_download, validate_create_request};
    use base64::Engine;
    use domus_common::types::SharedLinkType;
    use uuid::Uuid;

    #[test]
    fn decode_shared_link_key_accepts_immich_base64url_and_legacy_hex() {
        let key = [7u8; 32];
        let base64url = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(key);
        assert_eq!(decode_shared_link_key(&base64url).unwrap(), key);
        assert_eq!(decode_shared_link_key(&hex::encode(key)).unwrap(), key);
    }

    #[test]
    fn decode_shared_link_key_rejects_invalid_values() {
        assert!(decode_shared_link_key("not a key").is_err());
    }

    #[test]
    fn create_validation_matches_immich_shared_link_requirements() {
        assert!(validate_create_request(SharedLinkType::Album, None, &[]).is_err());
        assert!(validate_create_request(SharedLinkType::Album, Some(Uuid::nil()), &[]).is_ok());
        assert!(validate_create_request(SharedLinkType::Individual, None, &[]).is_err());
        assert!(validate_create_request(SharedLinkType::Individual, None, &[Uuid::nil()]).is_ok());
    }

    #[test]
    fn show_metadata_false_disables_download_like_immich() {
        assert!(effective_allow_download(true, true));
        assert!(!effective_allow_download(true, false));
        assert!(!effective_allow_download(false, true));
    }
}
