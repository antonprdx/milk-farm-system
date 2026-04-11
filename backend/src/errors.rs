use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable: {0}")]
    Unprocessable(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Unprocessable(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "Слишком много запросов".into(),
            ),
            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Внутренняя ошибка сервера".into(),
                )
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                match e {
                    sqlx::Error::Database(db_err) => {
                        if db_err.code().as_deref() == Some("23505") {
                            return (
                                StatusCode::CONFLICT,
                                axum::Json(ErrorBody {
                                    error: "Запись уже существует".into(),
                                }),
                            )
                                .into_response();
                        }
                        if db_err.code().as_deref() == Some("23503") {
                            return (
                                StatusCode::UNPROCESSABLE_ENTITY,
                                axum::Json(ErrorBody {
                                    error: "Связанная запись не найдена".into(),
                                }),
                            )
                                .into_response();
                        }
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Ошибка базы данных".into(),
                        )
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Ошибка базы данных".into(),
                    ),
                }
            }
        };

        (status, axum::Json(ErrorBody { error: message })).into_response()
    }
}
