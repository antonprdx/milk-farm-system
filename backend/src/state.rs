use std::sync::Arc;

use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

use crate::config::Config;

pub type AppState = Arc<AppStateInner>;

#[derive(Clone)]
pub struct AppStateInner {
    pub pool: PgPool,
    pub config: Config,
    pub lely_cancel: CancellationToken,
}
