//! Partner sharing: expose one user's timeline to another.

use domus_common::{Error, Result};
use domus_db::Repositories;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartnerDirection {
    SharedBy,
    SharedWith,
}

#[derive(Debug, Clone)]
pub struct PartnerWithUser {
    pub partner: domus_db::entities::Partner,
    pub user: domus_db::entities::User,
}

pub struct PartnerService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl PartnerService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        direction: PartnerDirection,
    ) -> Result<Vec<PartnerWithUser>> {
        let partners = self.repos.partner.list(user_id).await?;
        let mut response = Vec::new();
        for partner in partners {
            let shared_user_id = match direction {
                PartnerDirection::SharedBy if partner.shared_by_id == user_id => {
                    partner.shared_with_id
                }
                PartnerDirection::SharedWith if partner.shared_with_id == user_id => {
                    partner.shared_by_id
                }
                _ => continue,
            };
            response.push(PartnerWithUser {
                partner,
                user: self.repos.user.get(shared_user_id).await?,
            });
        }
        Ok(response)
    }

    pub async fn create(&self, shared_by: Uuid, shared_with: Uuid) -> Result<PartnerWithUser> {
        if self
            .repos
            .partner
            .get(shared_by, shared_with)
            .await?
            .is_some()
        {
            return Err(Error::BadRequest("Partner already exists".into()));
        }
        let partner = self.repos.partner.create(shared_by, shared_with).await?;
        Ok(PartnerWithUser {
            partner,
            user: self.repos.user.get(shared_with).await?,
        })
    }

    pub async fn remove(&self, shared_by: Uuid, shared_with: Uuid) -> Result<()> {
        if self
            .repos
            .partner
            .get(shared_by, shared_with)
            .await?
            .is_none()
        {
            return Err(Error::BadRequest("Partner not found".into()));
        }
        self.repos.partner.remove(shared_by, shared_with).await
    }

    pub async fn update_timeline(
        &self,
        shared_with: Uuid,
        shared_by: Uuid,
        in_timeline: bool,
    ) -> Result<PartnerWithUser> {
        if self
            .repos
            .partner
            .get(shared_by, shared_with)
            .await?
            .is_none()
        {
            return Err(Error::BadRequest("Partner not found".into()));
        }
        let partner = self
            .repos
            .partner
            .update_timeline(shared_by, shared_with, in_timeline)
            .await?;
        Ok(PartnerWithUser {
            partner,
            user: self.repos.user.get(shared_by).await?,
        })
    }
}

pub fn parse_partner_direction(value: &str) -> Result<PartnerDirection> {
    match value {
        "shared-by" => Ok(PartnerDirection::SharedBy),
        "shared-with" => Ok(PartnerDirection::SharedWith),
        _ => Err(Error::BadRequest(
            "Invalid option: expected one of \"shared-by\"|\"shared-with\"".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_partner_direction, PartnerDirection};

    #[test]
    fn parse_partner_direction_matches_immich_enum_values() {
        assert_eq!(
            parse_partner_direction("shared-by").unwrap(),
            PartnerDirection::SharedBy
        );
        assert_eq!(
            parse_partner_direction("shared-with").unwrap(),
            PartnerDirection::SharedWith
        );
        assert!(parse_partner_direction("invalid").is_err());
    }
}
