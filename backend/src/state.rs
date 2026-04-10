use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub pool: PgPool,
    pub config: Config,
}
