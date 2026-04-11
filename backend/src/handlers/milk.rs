use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::milk::{CreateMilkDayProduction, MilkFilter, UpdateMilkDayProduction};
use crate::models::pagination::paginated;
use crate::services::milk_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/milk/day-productions",
            get(list_productions).post(create_production),
        )
        .route(
            "/milk/day-productions/{id}",
            get(get_production)
                .put(update_production)
                .delete(delete_production),
        )
        .route("/milk/visits", get(list_visits))
        .route("/milk/quality", get(list_quality))
}

#[utoipa::path(
    get,
    path = "/api/v1/milk/day-productions",
    responses(
        (status = 200, description = "List of milk day productions", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(MilkFilter),
    security(("cookie_auth" = []))
)]
async fn list_productions(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<MilkFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || milk_service::list_productions(pool, f),
        || milk_service::count_productions(pool, f),
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/milk/day-productions",
    request_body = CreateMilkDayProduction,
    responses(
        (status = 201, description = "Production created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create_production(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateMilkDayProduction>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = milk_service::create_production(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    get,
    path = "/api/v1/milk/day-productions/{id}",
    responses(
        (status = 200, description = "Production found", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Production ID")),
    security(("cookie_auth" = []))
)]
async fn get_production(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = milk_service::get_production(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Запись о надое {} не найдена", id)))?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    put,
    path = "/api/v1/milk/day-productions/{id}",
    request_body = UpdateMilkDayProduction,
    responses(
        (status = 200, description = "Production updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Production ID")),
    security(("cookie_auth" = []))
)]
async fn update_production(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateMilkDayProduction>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = milk_service::update_production(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/milk/day-productions/{id}",
    responses(
        (status = 200, description = "Production deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Production ID")),
    security(("cookie_auth" = []))
)]
async fn delete_production(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    milk_service::delete_production(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/milk/visits",
    responses(
        (status = 200, description = "List of milk visits", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(MilkFilter),
    security(("cookie_auth" = []))
)]
async fn list_visits(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<MilkFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || milk_service::list_visits(pool, f),
        || milk_service::count_visits(pool, f),
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/milk/quality",
    responses(
        (status = 200, description = "List of milk quality records", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(MilkFilter),
    security(("cookie_auth" = []))
)]
async fn list_quality(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<MilkFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || milk_service::list_quality(pool, f),
        || milk_service::count_quality(pool, f),
    )
    .await
}
