use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::state::AppState;

const ISSUER: &str = "milk-farm";
const AUDIENCE: &str = "milk-farm-api";

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub must_change_password: bool,
    pub exp: usize,
    pub jti: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    pub iss: String,
    pub aud: String,
}

#[derive(Debug, Clone)]
pub struct AdminGuard;

#[derive(Debug, Clone)]
pub struct ClaimsAllowMustChange(pub Claims);

fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::default();
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[AUDIENCE]);
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::Unauthorized("Токен просрочен".into())
        }
        _ => AppError::Unauthorized("Недействительный токен".into()),
    })
}

async fn verify_and_check_revocation(
    token: &str,
    secret: &str,
    pool: &sqlx::PgPool,
) -> Result<Claims, AppError> {
    let claims = decode_token(token, secret)?;
    if crate::services::token_revocation_service::is_revoked(pool, &claims.jti).await? {
        return Err(AppError::Unauthorized("Токен отозван".into()));
    }
    let user_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
            .bind(&claims.sub)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;
    if !user_exists {
        return Err(AppError::Unauthorized(
            "Учётная запись деактивирована".into(),
        ));
    }
    Ok(claims)
}

impl FromRequestParts<AppState> for ClaimsAllowMustChange {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let token_result = extract_token(parts);
        let secret = state.config.jwt_secret.clone();
        let pool = state.pool.clone();
        async move {
            let token = token_result?;
            let claims = verify_and_check_revocation(&token, &secret, &pool).await?;
            Ok(ClaimsAllowMustChange(claims))
        }
    }
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
        jti: uuid::Uuid::new_v4().to_string(),
        iss: ISSUER.to_string(),
        aud: AUDIENCE.to_string(),
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
        jti: uuid::Uuid::new_v4().to_string(),
        iss: ISSUER.to_string(),
        aud: AUDIENCE.to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.into()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode_token(token, secret)
}

pub fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").map(|s| s.to_string()))
        .or_else(|| extract_token_from_cookies(headers))
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
        .ok_or_else(|| AppError::Unauthorized("Отсутствует токен".into()))
}

fn extract_refresh_token_from_cookies(headers: &HeaderMap) -> Option<String> {
    let cookie_str = headers.get("Cookie")?.to_str().ok()?;
    for part in cookie_str.split(';') {
        let trimmed = part.trim();
        if let Some(val) = trimmed.strip_prefix("refresh_token=") {
            return Some(val.to_string());
        }
    }
    None
}

pub fn extract_token_from_parts(parts: &Parts) -> Option<String> {
    extract_token_from_headers(&parts.headers)
}

pub fn extract_refresh_token_from_headers(headers: &HeaderMap) -> Option<String> {
    extract_refresh_token_from_cookies(headers)
}

impl FromRequestParts<AppState> for Claims {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let token_result = extract_token(parts);
        let secret = state.config.jwt_secret.clone();
        let pool = state.pool.clone();
        async move {
            let token = token_result?;
            let claims = verify_and_check_revocation(&token, &secret, &pool).await?;
            if claims.token_type.as_deref() != Some("access") {
                return Err(AppError::Unauthorized("Неверный тип токена".into()));
            }
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
        let pool = state.pool.clone();
        async move {
            let token = token_result?;
            let claims = verify_and_check_revocation(&token, &secret, &pool).await?;
            if claims.token_type.as_deref() != Some("access") {
                return Err(AppError::Unauthorized("Неверный тип токена".into()));
            }
            if claims.role != "admin" {
                return Err(AppError::Forbidden("Требуются права администратора".into()));
            }
            Ok(AdminGuard)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
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
