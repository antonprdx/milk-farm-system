use axum::extract::State;
use axum::http::HeaderValue;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{
    Claims, LoginRequest, RegisterRequest, create_access_token, create_refresh_token,
    extract_refresh_token_from_headers, extract_token_from_headers, verify_token,
};
use crate::middleware::rate_limit::{RateLimiter, extract_client_ip};
use crate::services::{system_settings_service, token_revocation_service, user_service};
use crate::state::AppState;

static LOGIN_LIMITER: std::sync::LazyLock<RateLimiter> =
    std::sync::LazyLock::new(|| RateLimiter::new(5, 60));
static REGISTER_LIMITER: std::sync::LazyLock<RateLimiter> =
    std::sync::LazyLock::new(|| RateLimiter::new(5, 60));

fn build_cookie(
    name: &str,
    value: &str,
    secure: bool,
    max_age: u64,
) -> Result<HeaderValue, AppError> {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!("{name}={value}; HttpOnly{secure_flag}; SameSite=Strict; Path=/; Max-Age={max_age}")
        .parse()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to build cookie header")))
}

fn set_auth_cookies(
    response: &mut Response,
    access_token: &str,
    refresh_token: &str,
    access_ttl: u64,
    refresh_ttl: u64,
    secure: bool,
) -> Result<(), AppError> {
    let headers = response.headers_mut();
    headers.insert(
        SET_COOKIE,
        build_cookie("token", access_token, secure, access_ttl)?,
    );
    headers.append(
        SET_COOKIE,
        build_cookie("refresh_token", refresh_token, secure, refresh_ttl)?,
    );
    Ok(())
}

fn set_clear_cookies(response: &mut Response, secure: bool) -> Result<(), AppError> {
    let headers = response.headers_mut();
    headers.insert(SET_COOKIE, build_cookie("token", "", secure, 0)?);
    headers.append(SET_COOKIE, build_cookie("refresh_token", "", secure, 0)?);
    Ok(())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/logout", post(logout))
        .route("/auth/refresh", post(refresh))
        .route("/health", get(health))
        .route("/healthz", get(liveness))
        .route("/readyz", get(readiness))
        .route("/stats", get(stats))
}

#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Health check", body = serde_json::Value)
    )
)]
async fn health(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();

    let lely_cfg = state.lely.get_config();
    let lely_status = if lely_cfg.enabled && !lely_cfg.base_url.is_empty() {
        let last_sync: Option<(String, String)> = sqlx::query_as(
            "SELECT status, TO_CHAR(last_synced_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') \
             FROM lely_sync_state WHERE entity_type = 'animals'",
        )
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten();
        json!({
            "enabled": true,
            "connected": last_sync.as_ref().map(|(s, _)| s == "success").unwrap_or(false),
            "last_sync": last_sync.map(|(_, dt)| dt),
        })
    } else {
        json!({ "enabled": false })
    };

    if db_ok {
        Ok(Json(
            json!({ "status": "ok", "db": "ok", "lely": lely_status }),
        ))
    } else {
        Ok(Json(
            json!({ "status": "degraded", "db": "error", "lely": lely_status }),
        ))
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/healthz",
    responses(
        (status = 200, description = "Service alive")
    )
)]
async fn liveness() -> Json<Value> {
    Json(json!({ "status": "alive" }))
}

#[utoipa::path(
    get,
    path = "/api/v1/readyz",
    responses(
        (status = 200, description = "Service ready"),
        (status = 503, description = "Service not ready")
    )
)]
async fn readiness(
    State(state): State<AppState>,
) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    if db_ok {
        Ok(Json(json!({ "status": "ready" })))
    } else {
        Err((
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "not_ready", "db": "error" })),
        ))
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful"),
        (status = 401, description = "Invalid credentials"),
        (status = 429, description = "Rate limited")
    )
)]
async fn login(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&headers, state.config.trust_proxy);
    LOGIN_LIMITER.check(&format!("login:{}", ip))?;
    req.validate()?;

    let user = user_service::find_by_username(&state.pool, &req.username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Неверные учётные данные".into()))?;

    let valid = bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.into()))?;

    if !valid {
        tracing::warn!(username = %req.username, ip = %ip, "Login failed: invalid password");
        return Err(AppError::Unauthorized("Неверные учётные данные".into()));
    }

    let ttl = system_settings_service::get_jwt_ttl(&state.pool)
        .await
        .unwrap_or_else(|_| crate::models::system_settings::JwtTtlSettings {
            jwt_access_ttl_secs: state.config.jwt_access_ttl_secs,
            jwt_refresh_ttl_secs: state.config.jwt_refresh_ttl_secs,
        });

    let access_token = create_access_token(
        &user.username,
        &user.role,
        user.must_change_password,
        &state.config.jwt_secret,
        ttl.jwt_access_ttl_secs,
    )?;

    let refresh_token = create_refresh_token(
        &user.username,
        &user.role,
        &state.config.jwt_secret,
        ttl.jwt_refresh_ttl_secs,
    )?;

    let mut response = axum::response::Json(json!({
        "username": user.username,
        "role": user.role,
        "must_change_password": user.must_change_password,
    }))
    .into_response();

    set_auth_cookies(
        &mut response,
        &access_token,
        &refresh_token,
        ttl.jwt_access_ttl_secs,
        ttl.jwt_refresh_ttl_secs,
        state.config.secure_cookies,
    )?;

    tracing::info!(username = %user.username, role = %user.role, ip = %ip, "User logged in");
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "Logged out")
    )
)]
async fn logout(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    if let Some(token_str) = extract_token_from_headers(&headers)
        && let Ok(claims) = verify_token(&token_str, &state.config.jwt_secret)
    {
        let exp =
            chrono::DateTime::from_timestamp(claims.exp as i64, 0).unwrap_or(chrono::Utc::now());
        if let Err(e) = token_revocation_service::revoke(&state.pool, &claims.jti, exp).await {
            tracing::warn!(error = %e, jti = %claims.jti, "Failed to revoke access token on logout");
        }
    }
    if let Some(refresh_str) = extract_refresh_token_from_headers(&headers)
        && let Ok(claims) = verify_token(&refresh_str, &state.config.jwt_secret)
    {
        let exp =
            chrono::DateTime::from_timestamp(claims.exp as i64, 0).unwrap_or(chrono::Utc::now());
        if let Err(e) = token_revocation_service::revoke(&state.pool, &claims.jti, exp).await {
            tracing::warn!(error = %e, jti = %claims.jti, "Failed to revoke refresh token on logout");
        }
    }

    let mut response = axum::response::Json(json!({ "message": "Выполнен выход" })).into_response();
    set_clear_cookies(&mut response, state.config.secure_cookies)?;

    if let Err(e) = token_revocation_service::cleanup_expired(&state.pool).await {
        tracing::warn!(error = %e, "Failed to cleanup expired tokens");
    }

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn register(
    _admin: crate::middleware::auth::AdminGuard,
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    let ip = extract_client_ip(&headers, state.config.trust_proxy);
    REGISTER_LIMITER.check(&format!("register:{}", ip))?;
    req.validate()?;

    let password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;

    user_service::create_user(&state.pool, &req.username, &password_hash, "user").await?;
    tracing::info!(username = %req.username, ip = %ip, "New user registered");

    Ok(Json(json!({ "message": "Пользователь создан" })))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed"),
        (status = 401, description = "Invalid refresh token")
    )
)]
async fn refresh(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let refresh_token_str = extract_refresh_token_from_headers(&headers)
        .ok_or_else(|| AppError::Unauthorized("Отсутствует refresh-токен".into()))?;

    let claims = verify_token(&refresh_token_str, &state.config.jwt_secret)?;

    if claims.token_type.as_deref() != Some("refresh") {
        return Err(AppError::Unauthorized("Неверный тип токена".into()));
    }

    if token_revocation_service::is_revoked(&state.pool, &claims.jti).await? {
        return Err(AppError::Unauthorized("Refresh-токен отозван".into()));
    }

    let exp = chrono::DateTime::from_timestamp(claims.exp as i64, 0).unwrap_or(chrono::Utc::now());
    token_revocation_service::revoke(&state.pool, &claims.jti, exp).await?;

    let user = user_service::find_by_username(&state.pool, &claims.sub)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Пользователь не найден".into()))?;

    let ttl = system_settings_service::get_jwt_ttl(&state.pool)
        .await
        .unwrap_or_else(|_| crate::models::system_settings::JwtTtlSettings {
            jwt_access_ttl_secs: state.config.jwt_access_ttl_secs,
            jwt_refresh_ttl_secs: state.config.jwt_refresh_ttl_secs,
        });

    let access_token = create_access_token(
        &user.username,
        &user.role,
        user.must_change_password,
        &state.config.jwt_secret,
        ttl.jwt_access_ttl_secs,
    )?;

    let new_refresh_token = create_refresh_token(
        &user.username,
        &user.role,
        &state.config.jwt_secret,
        ttl.jwt_refresh_ttl_secs,
    )?;

    let mut response = axum::response::Json(json!({
        "username": user.username,
        "role": user.role,
        "must_change_password": user.must_change_password,
    }))
    .into_response();

    set_auth_cookies(
        &mut response,
        &access_token,
        &new_refresh_token,
        ttl.jwt_access_ttl_secs,
        ttl.jwt_refresh_ttl_secs,
        state.config.secure_cookies,
    )?;

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/v1/stats",
    responses(
        (status = 200, description = "Dashboard stats", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn stats(_claims: Claims, State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let s = crate::services::stats_service::dashboard_stats(&state.pool).await?;
    Ok(Json(json!({
        "total_animals": s.total_animals,
        "total_females": s.total_females,
        "milk_today": s.milk_today,
        "in_heat": s.in_heat,
        "pregnant": s.pregnant,
    })))
}
