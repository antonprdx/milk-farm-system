use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::lely::client::LelyClient;
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
    pub password_set: bool,
    pub farm_key_set: bool,
    pub sync_interval_secs: u64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateLelyConfig {
    pub enabled: Option<bool>,
    pub base_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub farm_key: Option<String>,
    pub sync_interval_secs: Option<u64>,
}

impl UpdateLelyConfig {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        if let Some(ref v) = self.base_url
            && !v.is_empty()
            && !v.starts_with("http://")
            && !v.starts_with("https://")
        {
            return Err(AppError::BadRequest(
                "URL должен начинаться с http:// или https://".into(),
            ));
        }
        if let Some(ref v) = self.username {
            required_non_empty(v, "Имя пользователя Lely")?;
        }
        if let Some(v) = self.sync_interval_secs {
            range_u64(v, 60, 86400, "Интервал синхронизации (сек)")?;
        }
        Ok(())
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/lely/status", get(status))
        .route("/lely/sync", post(trigger_sync))
        .route("/lely/config", get(get_config).put(update_config))
        .route("/lely/test-connection", post(test_connection))
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
    let row = service::get_config_masked(&state.pool).await?;
    Ok(Json(LelyConfigResponse {
        enabled: row.enabled,
        base_url: row.base_url,
        username: row.username,
        password_set: !row.password_encrypted.is_empty(),
        farm_key_set: !row.farm_key_encrypted.is_empty(),
        sync_interval_secs: row.sync_interval_secs as u64,
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/lely/config",
    request_body = UpdateLelyConfig,
    responses(
        (status = 200, description = "Config updated", body = Value),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn update_config(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(body): Json<UpdateLelyConfig>,
) -> Result<Json<Value>, AppError> {
    body.validate()?;
    let mut current = state.lely.get_config();

    if let Some(v) = body.enabled {
        current.enabled = v;
    }
    if let Some(v) = body.base_url {
        current.base_url = v;
    }
    if let Some(v) = body.username {
        current.username = v;
    }
    if let Some(v) = body.password {
        current.password = v;
    }
    if let Some(v) = body.farm_key {
        current.farm_key = v;
    }
    if let Some(v) = body.sync_interval_secs {
        current.sync_interval_secs = v;
    }

    service::save_config(&state.pool, &current, &state.config.lely_encryption_key).await?;

    let _old_cancel = state.lely.set_config_and_restart_cancel(current.clone());

    if current.enabled {
        crate::lely::sync::start_sync_scheduler(state.clone());
        tracing::info!("Планировщик Lely перезапущен с новыми настройками");
    }

    Ok(Json(json!({ "message": "Настройки Lely сохранены" })))
}

#[utoipa::path(
    post,
    path = "/api/v1/lely/test-connection",
    responses(
        (status = 200, description = "Connection test result", body = Value),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn test_connection(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(body): Json<UpdateLelyConfig>,
) -> Result<Json<Value>, AppError> {
    let mut cfg = state.lely.get_config();
    if let Some(v) = body.base_url {
        cfg.base_url = v;
    }
    if let Some(v) = body.username {
        cfg.username = v;
    }
    if let Some(v) = body.password {
        cfg.password = v;
    }
    if let Some(v) = body.farm_key {
        cfg.farm_key = v;
    }

    if cfg.base_url.is_empty() || cfg.username.is_empty() || cfg.password.is_empty() {
        return Err(AppError::BadRequest(
            "URL, имя пользователя и пароль обязательны".into(),
        ));
    }

    let client = LelyClient::new(&cfg);
    match client.get_animals().await {
        Ok(animals) => Ok(Json(json!({
            "success": true,
            "message": format!("Подключение успешно. Найдено животных: {}", animals.len())
        }))),
        Err(e) => Ok(Json(json!({
            "success": false,
            "message": format!("Ошибка подключения: {}", e)
        }))),
    }
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
    let cfg = state.lely.get_config();
    if !cfg.enabled {
        return Err(AppError::BadRequest("Интеграция Lely не включена".into()));
    }

    let state_inner = std::sync::Arc::new(crate::state::AppStateInner {
        pool: state.pool.clone(),
        config: state.config.clone(),
        lely: state.lely.clone(),
    });

    tokio::spawn(async move {
        if let Err(e) = crate::lely::sync::run_sync(&state_inner).await {
            tracing::error!(error = %e, "Ошибка ручной синхронизации Lely");
        }
    });

    Ok(Json(json!({ "message": "Синхронизация Lely запущена" })))
}
