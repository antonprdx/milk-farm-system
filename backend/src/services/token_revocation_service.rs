use sqlx::PgPool;

use crate::errors::AppError;
use crate::services::retry::retry_db;

pub async fn revoke(pool: &PgPool, jti: &str, expires_at: chrono::DateTime<chrono::Utc>) -> Result<(), AppError> {
    let pool = pool.clone();
    let jti = jti.to_string();
    retry_db(move || {
        let pool = pool.clone();
        let jti = jti.clone();
        let expires_at = expires_at;
        async move {
            let jti_uuid: uuid::Uuid = jti.parse().map_err(|_| {
                AppError::Internal(anyhow::anyhow!("Invalid JTI format"))
            })?;
            sqlx::query(
                "INSERT INTO revoked_tokens (jti, expires_at) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(jti_uuid)
            .bind(expires_at)
            .execute(&pool)
            .await
            .map_err(AppError::Database)?;
            Ok::<_, AppError>(())
        }
    }).await
}

pub async fn is_revoked(pool: &PgPool, jti: &str) -> Result<bool, AppError> {
    let pool = pool.clone();
    let jti = jti.to_string();
    retry_db(move || {
        let pool = pool.clone();
        let jti = jti.clone();
        async move {
            let jti_uuid: uuid::Uuid = jti.parse().map_err(|_| {
                AppError::Internal(anyhow::anyhow!("Invalid JTI format"))
            })?;
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM revoked_tokens WHERE jti = $1)")
                    .bind(jti_uuid)
                    .fetch_one(&pool)
                    .await
                    .map_err(AppError::Database)?;
            Ok::<_, AppError>(exists)
        }
    }).await
}

pub async fn cleanup_expired(pool: &PgPool) -> Result<(), AppError> {
    let pool = pool.clone();
    retry_db(move || {
        let pool = pool.clone();
        async move {
            sqlx::query("DELETE FROM revoked_tokens WHERE expires_at < NOW()")
                .execute(&pool)
                .await
                .map_err(AppError::Database)?;
            Ok::<_, AppError>(())
        }
    }).await
}
