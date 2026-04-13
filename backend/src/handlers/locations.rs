use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::location::{CreateLocation, UpdateLocation};
use crate::models::pagination::simple_list;
use crate::services::location_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/locations", get(list).post(create))
        .route("/locations/{id}", put(update).delete(remove))
}

#[utoipa::path(
    get,
    path = "/api/v1/locations",
    responses(
        (status = 200, description = "List of locations", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn list(_claims: Claims, State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    simple_list(location_service::list(pool), location_service::count(pool)).await
}

#[utoipa::path(
    post,
    path = "/api/v1/locations",
    request_body = CreateLocation,
    responses(
        (status = 201, description = "Location created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateLocation>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = location_service::create(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    put,
    path = "/api/v1/locations/{id}",
    request_body = UpdateLocation,
    responses(
        (status = 200, description = "Location updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Location ID")),
    security(("cookie_auth" = []))
)]
async fn update(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateLocation>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = location_service::update(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/locations/{id}",
    responses(
        (status = 200, description = "Location deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Location ID")),
    security(("cookie_auth" = []))
)]
async fn remove(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    location_service::delete(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}
