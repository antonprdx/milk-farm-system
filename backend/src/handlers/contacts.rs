use axum::extract::{Path, Query, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::contact::{ContactFilter, CreateContact, UpdateContact};
use crate::models::pagination::paginated;
use crate::services::contact_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/contacts", get(list).post(create))
        .route("/contacts/{id}", put(update).delete(remove))
}

#[utoipa::path(
    get,
    path = "/api/v1/contacts",
    responses(
        (status = 200, description = "List of contacts", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(ContactFilter),
    security(("cookie_auth" = []))
)]
async fn list(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ContactFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || contact_service::list(pool, f),
        || contact_service::count(pool),
    )
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/contacts",
    request_body = CreateContact,
    responses(
        (status = 201, description = "Contact created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateContact>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = contact_service::create(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    put,
    path = "/api/v1/contacts/{id}",
    request_body = UpdateContact,
    responses(
        (status = 200, description = "Contact updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Contact ID")),
    security(("cookie_auth" = []))
)]
async fn update(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateContact>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = contact_service::update(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/contacts/{id}",
    responses(
        (status = 200, description = "Contact deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Contact ID")),
    security(("cookie_auth" = []))
)]
async fn remove(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    contact_service::delete(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}
