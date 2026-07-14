//! Domain layer: business logic, one service per feature area. Services own
//! authorization decisions and orchestration; repositories own SQL; the API
//! crate owns HTTP shapes.

pub mod services;

use domus_db::Repositories;
use domus_jobs::PgJobQueue;
use domus_media::storage::StorageCore;
use std::sync::Arc;

/// Service bundle handed to the API layer as shared state.
#[derive(Clone)]
pub struct Services {
    pub auth: Arc<services::auth::AuthService>,
    pub user: Arc<services::user::UserService>,
    pub asset: Arc<services::asset::AssetService>,
    pub asset_media: Arc<services::asset_media::AssetMediaService>,
    pub album: Arc<services::album::AlbumService>,
    pub timeline: Arc<services::timeline::TimelineService>,
    pub search: Arc<services::search::SearchService>,
    pub memory: Arc<services::memory::MemoryService>,
    pub partner: Arc<services::partner::PartnerService>,
    pub shared_link: Arc<services::shared_link::SharedLinkService>,
    pub tag: Arc<services::tag::TagService>,
    pub stack: Arc<services::stack::StackService>,
    pub library: Arc<services::library::LibraryService>,
    pub activity: Arc<services::activity::ActivityService>,
    pub notification: Arc<services::notification::NotificationService>,
    pub trash: Arc<services::trash::TrashService>,
    pub sync: Arc<services::sync::SyncService>,
    pub server: Arc<services::server::ServerService>,
    pub job_admin: Arc<services::job_admin::JobAdminService>,
}

impl Services {
    pub fn new(repos: Repositories, queue: PgJobQueue, storage: StorageCore) -> Self {
        Self {
            auth: Arc::new(services::auth::AuthService::new(repos.clone())),
            user: Arc::new(services::user::UserService::new(repos.clone())),
            asset: Arc::new(services::asset::AssetService::new(repos.clone(), queue.clone())),
            asset_media: Arc::new(services::asset_media::AssetMediaService::new(
                repos.clone(),
                queue.clone(),
                storage.clone(),
            )),
            album: Arc::new(services::album::AlbumService::new(repos.clone())),
            timeline: Arc::new(services::timeline::TimelineService::new(repos.clone())),
            search: Arc::new(services::search::SearchService::new(repos.clone())),
            memory: Arc::new(services::memory::MemoryService::new(repos.clone())),
            partner: Arc::new(services::partner::PartnerService::new(repos.clone())),
            shared_link: Arc::new(services::shared_link::SharedLinkService::new(repos.clone())),
            tag: Arc::new(services::tag::TagService::new(repos.clone())),
            stack: Arc::new(services::stack::StackService::new(repos.clone())),
            library: Arc::new(services::library::LibraryService::new(repos.clone(), queue.clone())),
            activity: Arc::new(services::activity::ActivityService::new(repos.clone())),
            notification: Arc::new(services::notification::NotificationService::new(repos.clone())),
            trash: Arc::new(services::trash::TrashService::new(repos.clone(), queue.clone())),
            sync: Arc::new(services::sync::SyncService::new(repos.clone())),
            server: Arc::new(services::server::ServerService::new(repos.clone())),
            job_admin: Arc::new(services::job_admin::JobAdminService::new(queue)),
        }
    }
}
