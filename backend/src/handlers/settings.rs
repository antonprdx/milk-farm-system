use axum::extract::{Path, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, Claims, ClaimsAllowMustChange, RegisterRequest};
use crate::models::preferences::{UpdatePreferences, UserPreferences};
use crate::models::system_settings::{
    AlertThresholds, JwtTtlSettings, SystemInfo, UpdateAlertThresholds, UpdateJwtTtl,
};
use crate::models::user::UserPublic;
use crate::services::{
    preferences_service, system_settings_service, token_revocation_service, user_service,
};
use crate::state::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

impl ChangePasswordRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        crate::validation::required_non_empty(&self.old_password, "Старый пароль")?;
        crate::validation::password(&self.new_password)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/settings/users", get(list_users).post(create_user))
        .route("/settings/users/{id}", delete(delete_user))
        .route("/settings/users/{id}/role", put(update_role))
        .route("/settings/password", post(change_password))
        .route(
            "/settings/preferences",
            get(get_preferences).put(update_preferences),
        )
        .route("/settings/system-info", get(system_info))
        .route("/settings/jwt-ttl", get(get_jwt_ttl).put(update_jwt_ttl))
        .route(
            "/settings/alert-thresholds",
            get(get_alert_thresholds).put(update_alert_thresholds),
        )
        .route("/settings/backup", get(backup_database))
}

async fn resolve_user_id(pool: &sqlx::PgPool, username: &str) -> Result<i32, AppError> {
    user_service::find_by_username(pool, username)
        .await?
        .map(|u| u.id)
        .ok_or_else(|| AppError::NotFound("Пользователь не найден".into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/users",
    responses(
        (status = 200, description = "List of users", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn list_users(
    _admin: AdminGuard,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let users = user_service::list_users(&state.pool).await?;
    let public_users: Vec<UserPublic> = users.into_iter().map(UserPublic::from).collect();
    Ok(Json(json!({ "data": public_users })))
}

#[utoipa::path(
    post,
    path = "/api/v1/settings/users",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create_user(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;
    user_service::create_user(&state.pool, &req.username, &hash, "user").await?;
    Ok(Json(json!({ "message": "Пользователь создан" })))
}

#[utoipa::path(
    post,
    path = "/api/v1/settings/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn change_password(
    claims: ClaimsAllowMustChange,
    _headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let user = user_service::find_by_username(&state.pool, &claims.0.sub)
        .await?
        .ok_or_else(|| AppError::NotFound("Пользователь не найден".into()))?;
    let valid = bcrypt::verify(&req.old_password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.into()))?;
    if !valid {
        return Err(AppError::Unauthorized("Неверный старый пароль".into()));
    }
    let new_hash = bcrypt::hash(&req.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;
    user_service::update_password_and_clear_flag(&state.pool, user.id, &new_hash).await?;

    let exp =
        chrono::DateTime::from_timestamp(claims.0.exp as i64, 0).unwrap_or(chrono::Utc::now());
    if let Err(e) = token_revocation_service::revoke(&state.pool, &claims.0.jti, exp).await {
        tracing::warn!(error = %e, jti = %claims.0.jti, "Failed to revoke token after password change");
    }

    Ok(Json(json!({ "message": "Пароль изменён" })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/settings/users/{id}",
    responses(
        (status = 200, description = "User deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "User ID")),
    security(("cookie_auth" = []))
)]
async fn delete_user(
    _admin: AdminGuard,
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let target = user_service::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Пользователь не найден".into()))?;
    if target.username == claims.sub {
        return Err(AppError::Forbidden(
            "Нельзя удалить свой собственный аккаунт".into(),
        ));
    }
    user_service::delete_user(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Пользователь удалён" })))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/users/{id}/role",
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "User ID")),
    security(("cookie_auth" = []))
)]
async fn update_role(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateRoleRequest>,
) -> Result<Json<Value>, AppError> {
    if body.role != "admin" && body.role != "user" {
        return Err(AppError::BadRequest(
            "Роль должна быть 'admin' или 'user'".into(),
        ));
    }
    user_service::update_role(&state.pool, id, &body.role).await?;
    Ok(Json(json!({ "message": "Роль обновлена" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/preferences",
    responses(
        (status = 200, description = "User preferences", body = UserPreferences),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn get_preferences(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<UserPreferences>, AppError> {
    let user_id = resolve_user_id(&state.pool, &claims.sub).await?;
    let prefs = preferences_service::get(&state.pool, user_id).await?;
    Ok(Json(prefs))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/preferences",
    request_body = UpdatePreferences,
    responses(
        (status = 200, description = "Updated preferences", body = UserPreferences),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn update_preferences(
    claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<UpdatePreferences>,
) -> Result<Json<UserPreferences>, AppError> {
    let user_id = resolve_user_id(&state.pool, &claims.sub).await?;
    let prefs = preferences_service::update(&state.pool, user_id, &req).await?;
    Ok(Json(prefs))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/system-info",
    responses(
        (status = 200, description = "System information", body = SystemInfo),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn system_info(
    _claims: AdminGuard,
    State(state): State<AppState>,
) -> Result<Json<SystemInfo>, AppError> {
    let info = system_settings_service::get_system_info(&state.pool).await?;
    Ok(Json(info))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/jwt-ttl",
    responses(
        (status = 200, description = "JWT TTL settings", body = JwtTtlSettings),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn get_jwt_ttl(
    _claims: AdminGuard,
    State(state): State<AppState>,
) -> Result<Json<JwtTtlSettings>, AppError> {
    let ttl = system_settings_service::get_jwt_ttl(&state.pool).await?;
    Ok(Json(ttl))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/jwt-ttl",
    request_body = UpdateJwtTtl,
    responses(
        (status = 200, description = "Updated JWT TTL settings", body = JwtTtlSettings),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn update_jwt_ttl(
    _claims: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<UpdateJwtTtl>,
) -> Result<Json<JwtTtlSettings>, AppError> {
    let ttl = system_settings_service::update_jwt_ttl(&state.pool, &req).await?;
    Ok(Json(ttl))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/alert-thresholds",
    responses(
        (status = 200, description = "Alert thresholds", body = AlertThresholds),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn get_alert_thresholds(
    _claims: AdminGuard,
    State(state): State<AppState>,
) -> Result<Json<AlertThresholds>, AppError> {
    let thresholds = system_settings_service::get_alert_thresholds(&state.pool).await?;
    Ok(Json(thresholds))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/alert-thresholds",
    request_body = UpdateAlertThresholds,
    responses(
        (status = 200, description = "Updated alert thresholds", body = AlertThresholds),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn update_alert_thresholds(
    _claims: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<UpdateAlertThresholds>,
) -> Result<Json<AlertThresholds>, AppError> {
    req.validate()?;
    let thresholds = system_settings_service::update_alert_thresholds(&state.pool, &req).await?;
    Ok(Json(thresholds))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/backup",
    responses(
        (status = 200, description = "Database backup SQL file"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn backup_database(
    _claims: AdminGuard,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let data = system_settings_service::generate_backup(&state.pool).await?;
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/sql"));
    let filename = format!(
        "milk_farm_backup_{}.sql",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    headers.insert(
        CONTENT_DISPOSITION,
        format!("attachment; filename=\"{filename}\"")
            .parse()
            .unwrap(),
    );
    Ok((headers, data))
}
