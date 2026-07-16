use crate::entities::Notification;
use crate::PgPool;
use domus_common::{Error, Result};
#[allow(unused_imports)]
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationRepository {
    #[allow(dead_code)]
    pool: PgPool,
}

impl NotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_user(
        &self,
        _user_id: Uuid,
        _unread_only: bool,
    ) -> Result<Vec<Notification>> {
        Err(Error::NotImplemented(
            "NotificationRepository::list_for_user",
        ))
    }

    pub async fn mark_read(&self, _ids: &[Uuid]) -> Result<()> {
        Err(Error::NotImplemented("NotificationRepository::mark_read"))
    }
}
