use std::sync::Arc;

use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

use crate::config::{Config, LelyConfig};

pub type AppState = Arc<AppStateInner>;

pub struct LelyRuntime {
    pub config: Arc<std::sync::RwLock<LelyConfig>>,
    pub cancel: Arc<std::sync::RwLock<CancellationToken>>,
}

impl LelyRuntime {
    pub fn new(config: LelyConfig) -> Self {
        Self {
            config: Arc::new(std::sync::RwLock::new(config)),
            cancel: Arc::new(std::sync::RwLock::new(CancellationToken::new())),
        }
    }

    pub fn get_config(&self) -> LelyConfig {
        match self.config.read() {
            Ok(guard) => guard.clone(),
            Err(e) => {
                tracing::error!("Lely config lock poisoned: {}", e);
                e.into_inner().clone()
            }
        }
    }

    pub fn set_config_and_restart_cancel(&self, cfg: LelyConfig) -> CancellationToken {
        {
            match self.config.write() {
                Ok(mut lock) => *lock = cfg,
                Err(e) => {
                    tracing::error!("Lely config lock poisoned: {}", e);
                    *e.into_inner() = cfg;
                }
            }
        }
        let old = {
            match self.cancel.write() {
                Ok(mut lock) => {
                    let old = lock.clone();
                    *lock = CancellationToken::new();
                    old
                }
                Err(e) => {
                    tracing::error!("Lely cancel lock poisoned: {}", e);
                    let mut lock = e.into_inner();
                    let old = lock.clone();
                    *lock = CancellationToken::new();
                    old
                }
            }
        };
        old.cancel();
        match self.cancel.read() {
            Ok(lock) => lock.clone(),
            Err(e) => {
                tracing::error!("Lely cancel read lock poisoned: {}", e);
                e.into_inner().clone()
            }
        }
    }
}

#[derive(Clone)]
pub struct AppStateInner {
    pub pool: PgPool,
    pub config: Config,
    pub lely: Arc<LelyRuntime>,
}
