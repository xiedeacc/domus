//! In-app notifications (backed by the notifications table, pushed over the socket.io channel).

#[allow(unused_imports)]
use domus_common::{Error, Result};
use domus_db::Repositories;
#[allow(unused_imports)]
use uuid::Uuid;

pub struct NotificationService {
    #[allow(dead_code)]
    repos: Repositories,
}

impl NotificationService {
    pub fn new(repos: Repositories) -> Self {
        Self { repos }
    }

    pub async fn list(&self, user_id: Uuid, unread_only: bool) -> Result<Vec<domus_db::entities::Notification>> {
        self.repos.notification.list_for_user(user_id, unread_only).await
    }

    pub async fn mark_read(&self, ids: &[Uuid]) -> Result<()> {
        self.repos.notification.mark_read(ids).await
    }
}
