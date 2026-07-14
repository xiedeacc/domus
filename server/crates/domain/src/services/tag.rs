//! Hierarchical tags (value = "a/b/c" path).

use domus_common::{Error, Result};
use domus_db::Repositories;
use uuid::Uuid;

pub struct TagService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl TagService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<domus_db::entities::Tag>> {
        self.repos.tag.list_for_user(user_id).await
    }

    pub async fn upsert(&self, user_id: Uuid, value: &str) -> Result<domus_db::entities::Tag> {
        let values = tag_path_values(value)?;
        let mut parent_id = None;
        let mut tag = None;

        for value in values {
            let upserted = self
                .repos
                .tag
                .upsert_value(user_id, &value, parent_id)
                .await?;
            parent_id = Some(upserted.id);
            tag = Some(upserted);
        }

        tag.ok_or_else(|| Error::BadRequest("tag value is required".into()))
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        parent_id: Option<Uuid>,
        color: Option<&str>,
    ) -> Result<domus_db::entities::Tag> {
        let value = if let Some(parent_id) = parent_id {
            let parent = self.repos.tag.get(parent_id).await?;
            format!("{}/{}", parent.value, name)
        } else {
            name.to_owned()
        };
        self.repos
            .tag
            .create(user_id, &value, parent_id, color)
            .await
    }

    pub async fn get(&self, id: Uuid) -> Result<domus_db::entities::Tag> {
        self.repos.tag.get(id).await
    }

    pub async fn update_color(
        &self,
        id: Uuid,
        color: Option<&str>,
    ) -> Result<domus_db::entities::Tag> {
        self.repos.tag.update_color(id, color).await
    }

    pub async fn tag_assets(&self, tag_id: Uuid, asset_ids: &[Uuid]) -> Result<u64> {
        self.repos.tag.tag_assets(tag_id, asset_ids).await
    }

    pub async fn untag_assets(&self, tag_id: Uuid, asset_ids: &[Uuid]) -> Result<u64> {
        self.repos.tag.untag_assets(tag_id, asset_ids).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        self.repos.tag.delete(id).await
    }
}

fn tag_path_values(path: &str) -> Result<Vec<String>> {
    let parts: Vec<_> = path
        .split('/')
        .filter(|part| !part.trim().is_empty())
        .map(str::trim)
        .collect();

    if parts.is_empty() {
        return Err(Error::BadRequest("tag value is required".into()));
    }

    let mut values = Vec::with_capacity(parts.len());
    let mut current = String::new();
    for part in parts {
        if !current.is_empty() {
            current.push('/');
        }
        current.push_str(part);
        values.push(current.clone());
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::tag_path_values;

    #[test]
    fn tag_path_values_upserts_nested_tags_like_immich() {
        assert_eq!(
            tag_path_values("Parent/Child/Leaf").unwrap(),
            vec!["Parent", "Parent/Child", "Parent/Child/Leaf"]
        );
    }

    #[test]
    fn tag_path_values_ignores_leading_and_trailing_slashes() {
        assert_eq!(
            tag_path_values("/Parent/Child/").unwrap(),
            vec!["Parent", "Parent/Child"]
        );
    }

    #[test]
    fn tag_path_values_rejects_empty_paths() {
        assert!(tag_path_values("///").is_err());
    }
}
