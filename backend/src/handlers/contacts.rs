use axum::extract::{Path, Query, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::contact::{ContactFilter, CreateContact, UpdateContact};
use crate::services::contact_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/contacts", get(list).post(create))
        .route("/contacts/{id}", put(update).delete(remove))
}

async fn list(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ContactFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = contact_service::list(&state.pool, &filter).await?;
    let total = contact_service::count(&state.pool).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}

async fn create(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateContact>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = contact_service::create(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn update(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateContact>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = contact_service::update(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

async fn remove(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    contact_service::delete(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}
