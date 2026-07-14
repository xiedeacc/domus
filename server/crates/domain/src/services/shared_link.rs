//! Public share links (key or slug in query string).

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
                let bytes = hex::decode(k).map_err(|_| Error::BadRequest("invalid key".into()))?;
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
        allow_download: bool,
        show_exif: bool,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<SharedLink> {
        let mut key = vec![0; 32];
        rand::rng().fill_bytes(&mut key);
        let link = SharedLink {
            id: Uuid::new_v4(),
            user_id,
            key,
            slug: None,
            link_type,
            album_id,
            description,
            password: None,
            allow_upload: false,
            allow_download,
            show_exif,
            expires_at,
            created_at: Utc::now(),
        };
        self.repos.shared_link.create(link, asset_ids).await
    }

    pub async fn update_options(
        &self,
        id: Uuid,
        allow_download: Option<bool>,
        show_exif: Option<bool>,
        description: Option<&str>,
    ) -> Result<SharedLink> {
        self.repos
            .shared_link
            .update_options(id, allow_download, show_exif, description)
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
