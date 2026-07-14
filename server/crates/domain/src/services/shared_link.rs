//! Public share links (key or slug in query string).

#[allow(unused_imports)]
use domus_common::{Error, Result};
use domus_db::Repositories;
#[allow(unused_imports)]
use uuid::Uuid;

pub struct SharedLinkService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl SharedLinkService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn resolve(&self, key: Option<&str>, slug: Option<&str>) -> Result<domus_db::entities::SharedLink> {
        match (key, slug) {
            (Some(k), _) => {
                let bytes = hex::decode(k).map_err(|_| Error::BadRequest("invalid key".into()))?;
                self.repos.shared_link.get_by_key(&bytes).await?.ok_or_else(|| Error::Unauthorized("invalid share key".into()))
            }
            (None, Some(s)) => self.repos.shared_link.get_by_slug(s).await?.ok_or_else(|| Error::Unauthorized("invalid share slug".into())),
            _ => Err(Error::BadRequest("missing key or slug".into())),
        }
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<domus_db::entities::SharedLink>> {
        self.repos.shared_link.list_for_user(user_id).await
    }
}
