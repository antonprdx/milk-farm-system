use std::sync::Arc;

use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

use crate::config::{Config, LelyConfig};
use crate::handlers::events::EventBus;
use crate::services::ml_cache::MlCache;
use crate::services::ml_client::MlClient;

pub type AppState = Arc<AppStateInner>;

pub struct LelyRuntime {
    pub config: Arc<tokio::sync::RwLock<LelyConfig>>,
    pub cancel: Arc<tokio::sync::RwLock<CancellationToken>>,
}

impl LelyRuntime {
    pub fn new(config: LelyConfig) -> Self {
        Self {
            config: Arc::new(tokio::sync::RwLock::new(config)),
            cancel: Arc::new(tokio::sync::RwLock::new(CancellationToken::new())),
        }
    }

    pub async fn get_config(&self) -> LelyConfig {
        self.config.read().await.clone()
    }

    pub async fn set_config_and_restart_cancel(&self, cfg: LelyConfig) -> CancellationToken {
        {
            let mut lock = self.config.write().await;
            *lock = cfg;
        }
        let old = {
            let mut lock = self.cancel.write().await;
            let old = lock.clone();
            *lock = CancellationToken::new();
            old
        };
        old.cancel();
        self.cancel.read().await.clone()
    }
}

#[derive(Clone)]
pub struct AppStateInner {
    pub pool: PgPool,
    pub config: Config,
    pub lely: Arc<LelyRuntime>,
    pub ml: Option<MlClient>,
    pub ml_cache: Option<MlCache>,
    pub redis: Option<redis::aio::ConnectionManager>,
    pub event_bus: Arc<EventBus>,
}
