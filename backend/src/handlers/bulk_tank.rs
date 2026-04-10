use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::bulk_tank::{BulkTankFilter, CreateBulkTankTest, UpdateBulkTankTest};
use crate::services::bulk_tank_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bulk-tank", get(list).post(create))
        .route("/bulk-tank/{id}", get(get_by_id).put(update).delete(remove))
}

async fn list(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<BulkTankFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = bulk_tank_service::list(&state.pool, &filter).await?;
    let total = bulk_tank_service::count(&state.pool, &filter).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}

async fn get_by_id(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = bulk_tank_service::get_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Bulk tank test {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

async fn create(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateBulkTankTest>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = bulk_tank_service::create(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn update(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateBulkTankTest>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = bulk_tank_service::update(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn remove(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    bulk_tank_service::delete(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}
