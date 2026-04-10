use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::fitness::FitnessFilter;
use crate::services::fitness_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/fitness/activities", get(list_activities))
        .route("/fitness/ruminations", get(list_ruminations))
}

async fn list_activities(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FitnessFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = fitness_service::list_activities(&state.pool, &filter).await?;
    let total = fitness_service::count_activities(&state.pool, &filter).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}

async fn list_ruminations(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FitnessFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = fitness_service::list_ruminations(&state.pool, &filter).await?;
    let total = fitness_service::count_ruminations(&state.pool, &filter).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}
