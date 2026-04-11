use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::fitness::FitnessFilter;
use crate::models::pagination::paginated;
use crate::services::fitness_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/fitness/activities", get(list_activities))
        .route("/fitness/ruminations", get(list_ruminations))
}

#[utoipa::path(
    get,
    path = "/api/v1/fitness/activities",
    responses(
        (status = 200, description = "List of activities", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(FitnessFilter),
    security(("cookie_auth" = []))
)]
async fn list_activities(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FitnessFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(filter.page, filter.per_page, || fitness_service::list_activities(pool, f), || fitness_service::count_activities(pool, f)).await
}

#[utoipa::path(
    get,
    path = "/api/v1/fitness/ruminations",
    responses(
        (status = 200, description = "List of ruminations", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(FitnessFilter),
    security(("cookie_auth" = []))
)]
async fn list_ruminations(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FitnessFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(filter.page, filter.per_page, || fitness_service::list_ruminations(pool, f), || fitness_service::count_ruminations(pool, f)).await
}
