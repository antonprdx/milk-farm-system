use axum::extract::{Path, State};
use axum::response::Json;
use serde::Deserialize;
use serde_json::json;

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SubscribeReq {
    pub channel_type: String,
    pub channel_token: String,
}

#[derive(Deserialize)]
pub struct CreateRuleReq {
    pub event_type: String,
    pub channel_id: Option<i32>,
}

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/notifications/channels", axum::routing::get(list_channels).post(create_channel))
        .route("/notifications/channels/{id}", axum::routing::delete(delete_channel))
        .route("/notifications/rules", axum::routing::get(list_rules).post(create_rule))
        .route("/notifications/rules/{id}", axum::routing::delete(delete_rule))
}

async fn resolve_uid(pool: &sqlx::PgPool, claims: &Claims) -> Result<i32, AppError> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT id FROM users WHERE username = $1",
    )
    .bind(&claims.sub)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;
    row.map(|r| r.0)
        .ok_or_else(|| AppError::Unauthorized("Пользователь не найден".into()))
}

async fn list_channels(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_uid(&state.pool, &claims).await?;
    let channels = crate::services::notification_service::list_channels(&state.pool, user_id).await?;
    Ok(Json(json!({ "data": channels })))
}

async fn create_channel(
    claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<SubscribeReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_uid(&state.pool, &claims).await?;
    let ch = crate::services::notification_service::create_channel(
        &state.pool,
        user_id,
        &crate::services::notification_service::CreateChannel {
            channel_type: req.channel_type,
            channel_token: req.channel_token,
        },
    )
    .await?;
    Ok(Json(json!({ "data": ch })))
}

async fn delete_channel(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_uid(&state.pool, &claims).await?;
    crate::services::notification_service::delete_channel(&state.pool, user_id, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}

async fn list_rules(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_uid(&state.pool, &claims).await?;
    let rules = crate::services::notification_service::list_rules(&state.pool, user_id).await?;
    Ok(Json(json!({ "data": rules })))
}

async fn create_rule(
    claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateRuleReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_uid(&state.pool, &claims).await?;
    crate::services::notification_service::create_rule(
        &state.pool,
        user_id,
        &crate::services::notification_service::CreateRule {
            event_type: req.event_type,
            channel_id: req.channel_id,
        },
    )
    .await?;
    Ok(Json(json!({ "message": "Правило создано" })))
}

async fn delete_rule(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_uid(&state.pool, &claims).await?;
    crate::services::notification_service::delete_rule(&state.pool, user_id, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}
