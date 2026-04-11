use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::bulk_tank::{BulkTankFilter, CreateBulkTankTest, UpdateBulkTankTest};
use crate::models::pagination::paginated;
use crate::services::bulk_tank_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bulk-tank", get(list).post(create))
        .route("/bulk-tank/{id}", get(get_by_id).put(update).delete(remove))
}

#[utoipa::path(
    get,
    path = "/api/v1/bulk-tank",
    responses(
        (status = 200, description = "List of bulk tank tests", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(BulkTankFilter),
    security(("cookie_auth" = []))
)]
async fn list(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<BulkTankFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || bulk_tank_service::list(pool, f),
        || bulk_tank_service::count(pool, f),
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/bulk-tank/{id}",
    responses(
        (status = 200, description = "Bulk tank test found", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Bulk tank test ID")),
    security(("cookie_auth" = []))
)]
async fn get_by_id(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = bulk_tank_service::get_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Запись анализа танка {} не найдена", id)))?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    post,
    path = "/api/v1/bulk-tank",
    request_body = CreateBulkTankTest,
    responses(
        (status = 201, description = "Bulk tank test created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateBulkTankTest>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = bulk_tank_service::create(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    put,
    path = "/api/v1/bulk-tank/{id}",
    request_body = UpdateBulkTankTest,
    responses(
        (status = 200, description = "Bulk tank test updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Bulk tank test ID")),
    security(("cookie_auth" = []))
)]
async fn update(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateBulkTankTest>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = bulk_tank_service::update(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/bulk-tank/{id}",
    responses(
        (status = 200, description = "Bulk tank test deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Bulk tank test ID")),
    security(("cookie_auth" = []))
)]
async fn remove(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    bulk_tank_service::delete(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}
