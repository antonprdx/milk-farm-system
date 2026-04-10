use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub must_change_password: bool,
    pub exp: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdminGuard;

#[derive(Debug, Clone)]
pub struct ClaimsAllowMustChange(pub Claims);

pub fn verify_token_allow_must_change(
    parts: &mut Parts,
    state: &AppState,
) -> Result<Claims, AppError> {
    let token_result = extract_token(parts);
    let secret = state.config.jwt_secret.clone();
    verify_token(&token_result?, &secret)
}

impl FromRequestParts<AppState> for ClaimsAllowMustChange {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = verify_token_allow_must_change(parts, state);
        async move {
            let claims = result?;
            Ok(ClaimsAllowMustChange(claims))
        }
    }
}

pub fn create_token(username: &str, role: &str, secret: &str) -> Result<String, AppError> {
    create_token_with_flag(username, role, false, secret)
}

pub fn create_token_with_flag(
    username: &str,
    role: &str,
    must_change_password: bool,
    secret: &str,
) -> Result<String, AppError> {
    create_access_token(username, role, must_change_password, secret, 24 * 3600)
}

pub fn create_access_token(
    username: &str,
    role: &str,
    must_change_password: bool,
    secret: &str,
    ttl_secs: u64,
) -> Result<String, AppError> {
    let exp = chrono::Utc::now().timestamp() as usize + ttl_secs as usize;
    let claims = Claims {
        sub: username.to_string(),
        role: role.to_string(),
        must_change_password,
        token_type: Some("access".to_string()),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.into()))
}

pub fn create_refresh_token(
    username: &str,
    role: &str,
    secret: &str,
    ttl_secs: u64,
) -> Result<String, AppError> {
    let exp = chrono::Utc::now().timestamp() as usize + ttl_secs as usize;
    let claims = Claims {
        sub: username.to_string(),
        role: role.to_string(),
        must_change_password: false,
        token_type: Some("refresh".to_string()),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.into()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::Unauthorized("Token expired".into())
        }
        _ => AppError::Unauthorized("Invalid token".into()),
    })
}

fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").map(|s| s.to_string()))
}

fn extract_token_from_cookies(headers: &HeaderMap) -> Option<String> {
    let cookie_str = headers.get("Cookie")?.to_str().ok()?;
    for part in cookie_str.split(';') {
        let trimmed = part.trim();
        if let Some(val) = trimmed.strip_prefix("token=") {
            return Some(val.to_string());
        }
    }
    None
}

fn extract_token(parts: &Parts) -> Result<String, AppError> {
    extract_token_from_headers(&parts.headers)
        .or_else(|| extract_token_from_cookies(&parts.headers))
        .ok_or_else(|| AppError::Unauthorized("Missing token".into()))
}

impl FromRequestParts<AppState> for Claims {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let token_result = extract_token(parts);
        let secret = state.config.jwt_secret.clone();
        async move {
            let token = token_result?;
            let claims = verify_token(&token, &secret)?;
            if claims.must_change_password {
                return Err(AppError::BadRequest(
                    "Необходимо сменить пароль перед использованием системы".into(),
                ));
            }
            Ok(claims)
        }
    }
}

impl FromRequestParts<AppState> for AdminGuard {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let token_result = extract_token(parts);
        let secret = state.config.jwt_secret.clone();
        async move {
            let token = token_result?;
            let claims = verify_token(&token, &secret)?;
            if claims.role != "admin" {
                return Err(AppError::Forbidden("Admin access required".into()));
            }
            Ok(AdminGuard)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl LoginRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        crate::validation::required_non_empty(&self.username, "Имя пользователя")?;
        crate::validation::required_non_empty(&self.password, "Пароль")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        crate::validation::username(&self.username)?;
        crate::validation::password(&self.password)?;
        Ok(())
    }
}
