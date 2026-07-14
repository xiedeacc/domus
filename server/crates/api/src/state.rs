use domus_common::Config;
use domus_domain::Services;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub services: Services,
}

impl AppState {
    pub fn new(config: Config, services: Services) -> Self {
        Self { config: Arc::new(config), services }
    }
}
