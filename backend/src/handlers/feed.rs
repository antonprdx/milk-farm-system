use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::feed::FeedFilter;
use crate::services::feed_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/feed/day-amounts", get(list_day_amounts))
        .route("/feed/visits", get(list_visits))
        .route("/feed/types", get(list_types))
        .route("/feed/groups", get(list_groups))
}

async fn list_day_amounts(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FeedFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = feed_service::list_day_amounts(&state.pool, &filter).await?;
    let total = feed_service::count_day_amounts(&state.pool, &filter).await?;
    Ok(Json(json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page })))
}

async fn list_visits(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FeedFilter>,
) -> Result<Json<Value>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = feed_service::list_visits(&state.pool, &filter).await?;
    let total = feed_service::count_visits(&state.pool, &filter).await?;
    Ok(Json(json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page })))
}

async fn list_types(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let data = feed_service::list_types(&state.pool).await?;
    let total = feed_service::count_types(&state.pool).await?;
    Ok(Json(json!({ "data": data, "total": total, "page": 1, "per_page": total })))
}

async fn list_groups(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let data = feed_service::list_groups(&state.pool).await?;
    let total = feed_service::count_groups(&state.pool).await?;
    Ok(Json(json!({ "data": data, "total": total, "page": 1, "per_page": total })))
}
