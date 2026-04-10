use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::milk::{CreateMilkDayProduction, MilkFilter, UpdateMilkDayProduction};
use crate::services::milk_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/milk/day-productions", get(list_productions).post(create_production))
        .route("/milk/day-productions/{id}", get(get_production).put(update_production).delete(delete_production))
        .route("/milk/visits", get(list_visits))
        .route("/milk/quality", get(list_quality))
}

async fn list_productions(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<MilkFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = milk_service::list_productions(&state.pool, &filter).await?;
    let total = milk_service::count_productions(&state.pool, &filter).await?;
    Ok(Json(json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page })))
}

async fn create_production(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateMilkDayProduction>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = milk_service::create_production(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn get_production(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = milk_service::get_production(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Production {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

async fn update_production(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateMilkDayProduction>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = milk_service::update_production(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn delete_production(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    milk_service::delete_production(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

async fn list_visits(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<MilkFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = milk_service::list_visits(&state.pool, &filter).await?;
    let total = milk_service::count_visits(&state.pool, &filter).await?;
    Ok(Json(json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page })))
}

async fn list_quality(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<MilkFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = milk_service::list_quality(&state.pool, &filter).await?;
    let total = milk_service::count_quality(&state.pool, &filter).await?;
    Ok(Json(json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page })))
}
