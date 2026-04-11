use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, Claims};
use crate::models::animal::{AnimalFilter, CreateAnimal, UpdateAnimal};
use crate::models::animal_stats::AnimalStats;
use crate::models::pagination::{Pagination, paginated};
use crate::models::timeline::{TimelineFilter, TimelineResponse};
use crate::services::{animal_service, animal_stats_service, timeline_service};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/animals", get(list).post(create))
        .route("/animals/{id}", get(get_by_id).put(update).delete(remove))
        .route("/animals/{id}/timeline", get(timeline))
        .route("/animals/{id}/stats", get(stats))
}

#[utoipa::path(
    get,
    path = "/api/v1/animals",
    responses(
        (status = 200, description = "List of animals", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(AnimalFilter),
    security(("cookie_auth" = []))
)]
async fn list(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<AnimalFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || animal_service::list(pool, f),
        || animal_service::count(pool, f),
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}",
    responses(
        (status = 200, description = "Animal found", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Animal ID")),
    security(("cookie_auth" = []))
)]
async fn get_by_id(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let animal = animal_service::get_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Животное с ID {} не найдено", id)))?;
    Ok(Json(json!({ "data": animal })))
}

#[utoipa::path(
    post,
    path = "/api/v1/animals",
    request_body = CreateAnimal,
    responses(
        (status = 201, description = "Animal created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateAnimal>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let animal = animal_service::create(&state.pool, &req).await?;
    Ok(Json(json!({ "data": animal })))
}

#[utoipa::path(
    put,
    path = "/api/v1/animals/{id}",
    request_body = UpdateAnimal,
    responses(
        (status = 200, description = "Animal updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Animal ID")),
    security(("cookie_auth" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/animals/{id}",
    responses(
        (status = 200, description = "Animal deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Animal ID")),
    security(("cookie_auth" = []))
)]
async fn remove(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    animal_service::delete(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}/timeline",
    responses(
        (status = 200, description = "Timeline events", body = TimelineResponse),
        (status = 404, description = "Animal not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = i32, Path, description = "Animal ID"),
        TimelineFilter
    ),
    security(("cookie_auth" = []))
)]
async fn timeline(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Query(filter): Query<TimelineFilter>,
) -> Result<Json<TimelineResponse>, AppError> {
    animal_service::ensure_exists(&state.pool, id).await?;
    let pag = Pagination::from_filter(filter.page, filter.per_page);
    let (data, total) = tokio::join!(
        timeline_service::list(&state.pool, id, filter.page, filter.per_page),
        timeline_service::count(&state.pool, id),
    );
    let data = data?;
    let total = total?;
    Ok(Json(TimelineResponse {
        data,
        total,
        page: pag.page,
        per_page: pag.per_page,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}/stats",
    responses(
        (status = 200, description = "Animal statistics", body = AnimalStats),
        (status = 404, description = "Animal not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Animal ID")),
    security(("cookie_auth" = []))
)]
async fn stats(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<AnimalStats>, AppError> {
    animal_service::ensure_exists(&state.pool, id).await?;
    let stats = animal_stats_service::get_animal_stats(&state.pool, id).await?;
    Ok(Json(stats))
}
