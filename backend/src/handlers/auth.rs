use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{Claims, LoginRequest, RegisterRequest};
use crate::state::AppState;

use crate::middleware::auth::{create_access_token, create_refresh_token, verify_token};
use crate::services::{system_settings_service, token_revocation_service, user_service};

struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

struct RateLimiter {
    max: u32,
    window_secs: u64,
    entries: Mutex<HashMap<String, RateLimitEntry>>,
}

impl RateLimiter {
    fn new(max: u32, window_secs: u64) -> Self {
        Self {
            max,
            window_secs,
            entries: Mutex::new(HashMap::new()),
        }
    }

    fn check(&self, key: &str) -> Result<(), AppError> {
        let mut map = self
            .entries
            .lock()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;
        let now = Instant::now();
        if let Some(entry) = map.get_mut(key) {
            if now.duration_since(entry.window_start).as_secs() > self.window_secs {
                entry.count = 1;
                entry.window_start = now;
            } else if entry.count >= self.max {
                return Err(AppError::RateLimited);
            } else {
                entry.count += 1;
            }
        } else {
            map.insert(
                key.to_string(),
                RateLimitEntry {
                    count: 1,
                    window_start: now,
                },
            );
        }
        map.retain(|_, v| now.duration_since(v.window_start).as_secs() <= self.window_secs * 2);
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref LOGIN_LIMITER: RateLimiter = RateLimiter::new(5, 60);
    static ref REGISTER_LIMITER: RateLimiter = RateLimiter::new(5, 60);
}

fn extract_client_ip(headers: &axum::http::HeaderMap, trust_proxy: bool) -> String {
    if trust_proxy {
        if let Some(forwarded) = headers.get("X-Forwarded-For")
            && let Ok(val) = forwarded.to_str()
            && let Some(ip) = val.split(',').next()
        {
            let ip = ip.trim().to_string();
            if !ip.is_empty() {
                return ip;
            }
        }
    }
    "unknown".to_string()
}

fn build_cookie(name: &str, value: &str, secure: bool, max_age: u64) -> Result<HeaderValue, AppError> {
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
    headers.insert(SET_COOKIE, build_cookie("token", access_token, secure, access_ttl)?);
    headers.append(SET_COOKIE, build_cookie("refresh_token", refresh_token, secure, refresh_ttl)?);
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
    if db_ok {
        Ok(Json(json!({ "status": "ok", "db": "ok" })))
    } else {
        Ok(Json(json!({ "status": "degraded", "db": "error" })))
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
async fn readiness(State(state): State<AppState>) -> Result<Json<Value>, (axum::http::StatusCode, Json<Value>)> {
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
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

    let valid = bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.into()))?;

    if !valid {
        tracing::warn!(username = %req.username, ip = %ip, "Login failed: invalid password");
        return Err(AppError::Unauthorized("Invalid credentials".into()));
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
    if let Some(token_str) = extract_token_str_from_headers(&headers) {
        if let Ok(claims) = verify_token(&token_str, &state.config.jwt_secret) {
            let exp = chrono::DateTime::from_timestamp(claims.exp as i64, 0)
                .unwrap_or(chrono::Utc::now());
            if let Err(e) = token_revocation_service::revoke(&state.pool, &claims.jti, exp).await {
                tracing::warn!(error = %e, jti = %claims.jti, "Failed to revoke access token on logout");
            }
        }
    }
    if let Some(refresh_str) = extract_refresh_token_from_cookies(&headers) {
        if let Ok(claims) = verify_token(&refresh_str, &state.config.jwt_secret) {
            let exp = chrono::DateTime::from_timestamp(claims.exp as i64, 0)
                .unwrap_or(chrono::Utc::now());
            if let Err(e) = token_revocation_service::revoke(&state.pool, &claims.jti, exp).await {
                tracing::warn!(error = %e, jti = %claims.jti, "Failed to revoke refresh token on logout");
            }
        }
    }

    let mut response = axum::response::Json(json!({ "message": "Logged out" })).into_response();
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

    Ok(Json(json!({ "message": "User created" })))
}

fn extract_refresh_token_from_cookies(headers: &axum::http::HeaderMap) -> Option<String> {
    let cookie_str = headers.get("Cookie")?.to_str().ok()?;
    for part in cookie_str.split(';') {
        let trimmed = part.trim();
        if let Some(val) = trimmed.strip_prefix("refresh_token=") {
            return Some(val.to_string());
        }
    }
    None
}

fn extract_token_str_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").map(|s| s.to_string()))
        .or_else(|| {
            let cookie_str = headers.get("Cookie")?.to_str().ok()?;
            for part in cookie_str.split(';') {
                let trimmed = part.trim();
                if let Some(val) = trimmed.strip_prefix("token=") {
                    return Some(val.to_string());
                }
            }
            None
        })
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
    let refresh_token_str = extract_refresh_token_from_cookies(&headers)
        .ok_or_else(|| AppError::Unauthorized("Missing refresh token".into()))?;

    let claims = verify_token(&refresh_token_str, &state.config.jwt_secret)?;

    if claims.token_type.as_deref() != Some("refresh") {
        return Err(AppError::Unauthorized("Invalid token type".into()));
    }

    if token_revocation_service::is_revoked(&state.pool, &claims.jti).await? {
        return Err(AppError::Unauthorized("Refresh token revoked".into()));
    }

    let exp = chrono::DateTime::from_timestamp(claims.exp as i64, 0)
        .unwrap_or(chrono::Utc::now());
    token_revocation_service::revoke(&state.pool, &claims.jti, exp).await?;

    let user = user_service::find_by_username(&state.pool, &claims.sub)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

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
