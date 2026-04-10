use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{Claims, LoginRequest, RegisterRequest};
use crate::services::user_service;
use crate::state::AppState;

use crate::middleware::auth::{create_access_token, create_refresh_token, verify_token};

struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

const RATE_LIMIT_MAX: u32 = 5;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/logout", post(logout))
        .route("/auth/refresh", post(refresh))
        .route("/health", get(health))
        .route("/stats", get(stats))
}

fn check_rate_limit(
    map: &Mutex<HashMap<String, RateLimitEntry>>,
    key: &str,
) -> Result<(), AppError> {
    let mut map = map
        .lock()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;
    let now = Instant::now();
    if let Some(entry) = map.get_mut(key) {
        if now.duration_since(entry.window_start).as_secs() > RATE_LIMIT_WINDOW_SECS {
            entry.count = 1;
            entry.window_start = now;
        } else if entry.count >= RATE_LIMIT_MAX {
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
    map.retain(|_, v| now.duration_since(v.window_start).as_secs() <= RATE_LIMIT_WINDOW_SECS * 2);
    Ok(())
}

fn extract_client_ip(headers: &axum::http::HeaderMap) -> String {
    if let Some(forwarded) = headers.get("X-Forwarded-For")
        && let Ok(val) = forwarded.to_str()
        && let Some(ip) = val.split(',').next()
    {
        let ip = ip.trim().to_string();
        if !ip.is_empty() {
            return ip;
        }
    }
    "unknown".to_string()
}

lazy_static::lazy_static! {
    static ref LOGIN_LIMITS: Mutex<HashMap<String, RateLimitEntry>> = Mutex::new(HashMap::new());
    static ref REGISTER_LIMITS: Mutex<HashMap<String, RateLimitEntry>> = Mutex::new(HashMap::new());
}

async fn health(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    if db_ok {
        Ok(Json(json!({ "status": "ok", "db": "ok" })))
    } else {
        Ok(Json(json!({ "status": "degraded", "db": "error" })))
    }
}

async fn login(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&headers);
    check_rate_limit(&LOGIN_LIMITS, &format!("login:{}", ip))?;
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

    let access_token = create_access_token(
        &user.username,
        &user.role,
        user.must_change_password,
        &state.config.jwt_secret,
        state.config.jwt_access_ttl_secs,
    )?;

    let refresh_token = create_refresh_token(
        &user.username,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_refresh_ttl_secs,
    )?;

    let secure_flag = if state.config.secure_cookies {
        "; Secure"
    } else {
        ""
    };
    let access_cookie = format!(
        "token={}; HttpOnly{}; SameSite=Strict; Path=/; Max-Age={}",
        access_token, secure_flag, state.config.jwt_access_ttl_secs
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly{}; SameSite=Strict; Path=/; Max-Age={}",
        refresh_token, secure_flag, state.config.jwt_refresh_ttl_secs
    );

    let mut response = axum::response::Json(json!({
        "username": user.username,
        "role": user.role,
        "must_change_password": user.must_change_password,
    }))
    .into_response();

    let headers = response.headers_mut();
    headers.insert(SET_COOKIE, access_cookie.parse().unwrap());
    headers.append(SET_COOKIE, refresh_cookie.parse().unwrap());

    tracing::info!(username = %user.username, role = %user.role, ip = %ip, "User logged in");
    Ok(response)
}

async fn logout(State(state): State<AppState>) -> Result<Response, AppError> {
    let secure_flag = if state.config.secure_cookies {
        "; Secure"
    } else {
        ""
    };
    let access_cookie = format!(
        "token=; HttpOnly{}; SameSite=Strict; Path=/; Max-Age=0",
        secure_flag
    );
    let refresh_cookie = format!(
        "refresh_token=; HttpOnly{}; SameSite=Strict; Path=/; Max-Age=0",
        secure_flag
    );
    let mut response = axum::response::Json(json!({ "message": "Logged out" })).into_response();
    let headers = response.headers_mut();
    headers.insert(SET_COOKIE, access_cookie.parse().unwrap());
    headers.append(SET_COOKIE, refresh_cookie.parse().unwrap());
    Ok(response)
}

async fn register(
    _admin: crate::middleware::auth::AdminGuard,
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    let ip = extract_client_ip(&headers);
    check_rate_limit(&REGISTER_LIMITS, &format!("register:{}", ip))?;
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

    let user = user_service::find_by_username(&state.pool, &claims.sub)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    let access_token = create_access_token(
        &user.username,
        &user.role,
        user.must_change_password,
        &state.config.jwt_secret,
        state.config.jwt_access_ttl_secs,
    )?;

    let new_refresh_token = create_refresh_token(
        &user.username,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_refresh_ttl_secs,
    )?;

    let secure_flag = if state.config.secure_cookies {
        "; Secure"
    } else {
        ""
    };
    let access_cookie = format!(
        "token={}; HttpOnly{}; SameSite=Strict; Path=/; Max-Age={}",
        access_token, secure_flag, state.config.jwt_access_ttl_secs
    );
    let refresh_cookie = format!(
        "refresh_token={}; HttpOnly{}; SameSite=Strict; Path=/; Max-Age={}",
        new_refresh_token, secure_flag, state.config.jwt_refresh_ttl_secs
    );

    let mut response = axum::response::Json(json!({
        "username": user.username,
        "role": user.role,
        "must_change_password": user.must_change_password,
    }))
    .into_response();

    let resp_headers = response.headers_mut();
    resp_headers.insert(SET_COOKIE, access_cookie.parse().unwrap());
    resp_headers.append(SET_COOKIE, refresh_cookie.parse().unwrap());

    Ok(response)
}

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
