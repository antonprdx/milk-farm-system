use axum::extract::{Path, Query, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, Claims};
use crate::models::pagination::paginated;
use crate::models::vet::{
    CreateVetRecord, CreateWeightRecord, UpdateVetRecord, VetRecordFilter, WeightRecordFilter,
};
use crate::services::vet_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/vet/records", get(list_records).post(create_record))
        .route(
            "/vet/records/{id}",
            get(get_record).put(update_record).delete(delete_record),
        )
        .route("/vet/weights", get(list_weights).post(create_weight))
        .route("/vet/weights/{id}", delete(delete_weight))
        .route("/vet/follow-ups", get(follow_ups))
        .route("/vet/withdrawals", get(withdrawals))
}

#[utoipa::path(
    get,
    path = "/api/v1/vet/records",
    params(VetRecordFilter),
    security(("cookie_auth" = []))
)]
async fn list_records(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<VetRecordFilter>,
) -> Result<Json<Value>, AppError> {
    paginated(
        filter.page,
        filter.per_page,
        || vet_service::list_vet_records(&state.pool, &filter),
        || vet_service::count_vet_records(&state.pool, &filter),
    )
    .await
}

async fn get_record(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let record = vet_service::get_vet_record(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Ветеринарная запись с ID {} не найдена", id)))?;
    Ok(Json(json!({ "data": record })))
}

async fn create_record(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateVetRecord>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let record = vet_service::create_vet_record(&state.pool, &req).await?;
    Ok(Json(json!({ "data": record })))
}

async fn update_record(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateVetRecord>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let record = vet_service::update_vet_record(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": record })))
}

async fn delete_record(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    vet_service::delete_vet_record(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/vet/weights",
    params(WeightRecordFilter),
    security(("cookie_auth" = []))
)]
async fn list_weights(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<WeightRecordFilter>,
) -> Result<Json<Value>, AppError> {
    paginated(
        filter.page,
        filter.per_page,
        || vet_service::list_weight_records(&state.pool, &filter),
        || vet_service::count_weight_records(&state.pool, &filter),
    )
    .await
}

async fn create_weight(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateWeightRecord>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let record = vet_service::create_weight_record(&state.pool, &req).await?;
    Ok(Json(json!({ "data": record })))
}

async fn delete_weight(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    vet_service::delete_weight_record(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}

#[derive(Debug, Deserialize)]
struct FollowUpQuery {
    days: Option<i32>,
}

async fn follow_ups(
    _claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<FollowUpQuery>,
) -> Result<Json<Value>, AppError> {
    let records = vet_service::upcoming_follow_ups(&state.pool, q.days.unwrap_or(7)).await?;
    Ok(Json(json!({ "data": records })))
}

async fn withdrawals(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let records = vet_service::active_withdrawals(&state.pool).await?;
    Ok(Json(json!({ "data": records })))
}
