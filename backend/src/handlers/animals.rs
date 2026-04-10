use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, Claims};
use crate::models::animal::{AnimalFilter, CreateAnimal, UpdateAnimal};
use crate::services::animal_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/animals", get(list).post(create))
        .route("/animals/{id}", get(get_by_id).put(update).delete(remove))
}

async fn list(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<AnimalFilter>,
) -> Result<Json<Value>, AppError> {
    let animals = animal_service::list(&state.pool, &filter).await?;
    let total = animal_service::count(&state.pool, &filter).await?;
    Ok(Json(json!({ "data": animals, "total": total })))
}

async fn get_by_id(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let animal = animal_service::get_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Animal {} not found", id)))?;
    Ok(Json(json!({ "data": animal })))
}

async fn create(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateAnimal>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let animal = animal_service::create(&state.pool, &req).await?;
    Ok(Json(json!({ "data": animal })))
}

async fn update(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateAnimal>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let animal = animal_service::update(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": animal })))
}

async fn remove(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    animal_service::delete(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}
