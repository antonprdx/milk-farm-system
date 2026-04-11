use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::lely::service;
use crate::middleware::auth::AdminGuard;
use crate::state::AppState;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LelySyncStatus {
    pub entity_type: String,
    pub last_synced_at: Option<String>,
    pub status: String,
    pub records_synced: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LelyConfigResponse {
    pub enabled: bool,
    pub base_url: String,
    pub username: String,
    pub farm_key_set: bool,
    pub sync_interval_secs: u64,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/lely/status", get(status))
        .route("/lely/sync", post(trigger_sync))
        .route("/lely/config", get(get_config))
}

#[utoipa::path(
    get,
    path = "/api/v1/lely/status",
    responses(
        (status = 200, description = "Lely sync status for all entities", body = Value),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn status(
    _admin: AdminGuard,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let states = service::get_all_sync_states(&state.pool).await?;
    let items: Vec<LelySyncStatus> = states
        .into_iter()
        .map(|s| LelySyncStatus {
            entity_type: s.entity_type,
            last_synced_at: s.last_synced_at.map(|dt| dt.to_rfc3339()),
            status: s.status,
            records_synced: s.records_synced,
            error_message: s.error_message,
        })
        .collect();
    Ok(Json(json!({ "data": items })))
}

#[utoipa::path(
    post,
    path = "/api/v1/lely/sync",
    responses(
        (status = 200, description = "Sync triggered", body = Value),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn trigger_sync(
    _admin: AdminGuard,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    if !state.config.lely.enabled {
        return Err(AppError::BadRequest("Интеграция Lely не включена".into()));
    }

    let pool = state.pool.clone();
    let cfg = state.config.clone();

    let state_inner = std::sync::Arc::new(crate::state::AppStateInner {
        pool,
        config: cfg,
        lely_cancel: state.lely_cancel.clone(),
    });

    tokio::spawn(async move {
        if let Err(e) = crate::lely::sync::run_sync(&state_inner).await {
            tracing::error!(error = %e, "Ошибка ручной синхронизации Lely");
        }
    });

    Ok(Json(json!({ "message": "Синхронизация Lely запущена" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/lely/config",
    responses(
        (status = 200, description = "Lely configuration", body = LelyConfigResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn get_config(
    _admin: AdminGuard,
    State(state): State<AppState>,
) -> Result<Json<LelyConfigResponse>, AppError> {
    Ok(Json(LelyConfigResponse {
        enabled: state.config.lely.enabled,
        base_url: state.config.lely.base_url.clone(),
        username: state.config.lely.username.clone(),
        farm_key_set: !state.config.lely.farm_key.is_empty(),
        sync_interval_secs: state.config.lely.sync_interval_secs,
    }))
}
