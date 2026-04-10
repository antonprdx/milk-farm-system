use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::reproduction::*;
use crate::services::reproduction_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/reproduction/calvings",
            get(list_calvings).post(create_calving),
        )
        .route(
            "/reproduction/calvings/{id}",
            get(get_calving).put(update_calving).delete(delete_calving),
        )
        .route(
            "/reproduction/inseminations",
            get(list_inseminations).post(create_insemination),
        )
        .route(
            "/reproduction/inseminations/{id}",
            get(get_insemination)
                .put(update_insemination)
                .delete(delete_insemination),
        )
        .route(
            "/reproduction/pregnancies",
            get(list_pregnancies).post(create_pregnancy),
        )
        .route(
            "/reproduction/pregnancies/{id}",
            get(get_pregnancy)
                .put(update_pregnancy)
                .delete(delete_pregnancy),
        )
        .route("/reproduction/heats", get(list_heats).post(create_heat))
        .route(
            "/reproduction/heats/{id}",
            get(get_heat).put(update_heat).delete(delete_heat),
        )
        .route(
            "/reproduction/dryoffs",
            get(list_dryoffs).post(create_dryoff),
        )
        .route(
            "/reproduction/dryoffs/{id}",
            get(get_dryoff).put(update_dryoff).delete(delete_dryoff),
        )
        .route("/reproduction/status", get(current_status))
}

async fn list_calvings(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = reproduction_service::list_calvings(&state.pool, &filter).await?;
    let total = reproduction_service::count_calvings(&state.pool, &filter).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}

async fn get_calving(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_calving(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Calving {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

async fn create_calving(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateCalving>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_calving(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn update_calving(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCalving>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_calving(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn delete_calving(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_calving(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

async fn list_inseminations(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = reproduction_service::list_inseminations(&state.pool, &filter).await?;
    let total = reproduction_service::count_inseminations(&state.pool, &filter).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}

async fn get_insemination(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_insemination(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Insemination {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

async fn create_insemination(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateInsemination>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_insemination(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn update_insemination(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateInsemination>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_insemination(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn delete_insemination(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_insemination(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

async fn list_pregnancies(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = reproduction_service::list_pregnancies(&state.pool, &filter).await?;
    let total = reproduction_service::count_pregnancies(&state.pool, &filter).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}

async fn get_pregnancy(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_pregnancy(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Pregnancy {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

async fn create_pregnancy(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreatePregnancy>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_pregnancy(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn update_pregnancy(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdatePregnancy>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_pregnancy(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn delete_pregnancy(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_pregnancy(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

async fn list_heats(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = reproduction_service::list_heats(&state.pool, &filter).await?;
    let total = reproduction_service::count_heats(&state.pool, &filter).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}

async fn get_heat(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_heat(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Heat {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

async fn create_heat(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateHeat>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_heat(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn update_heat(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateHeat>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_heat(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn delete_heat(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_heat(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

async fn list_dryoffs(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = reproduction_service::list_dryoffs(&state.pool, &filter).await?;
    let total = reproduction_service::count_dryoffs(&state.pool, &filter).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}

async fn get_dryoff(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_dryoff(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("DryOff {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

async fn create_dryoff(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateDryOff>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_dryoff(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn update_dryoff(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateDryOff>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_dryoff(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn delete_dryoff(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_dryoff(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

async fn current_status(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let data = reproduction_service::current_status(&state.pool).await?;
    Ok(Json(json!({ "data": data })))
}
