use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::feed::FeedFilter;
use crate::models::pagination::{paginated, simple_list};
use crate::services::feed_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/feed/day-amounts", get(list_day_amounts))
        .route("/feed/visits", get(list_visits))
        .route("/feed/types", get(list_types))
        .route("/feed/groups", get(list_groups))
}

#[utoipa::path(
    get,
    path = "/api/v1/feed/day-amounts",
    responses(
        (status = 200, description = "List of feed day amounts", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(FeedFilter),
    security(("cookie_auth" = []))
)]
async fn list_day_amounts(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FeedFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || feed_service::list_day_amounts(pool, f),
        || feed_service::count_day_amounts(pool, f),
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/feed/visits",
    responses(
        (status = 200, description = "List of feed visits", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(FeedFilter),
    security(("cookie_auth" = []))
)]
async fn list_visits(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FeedFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || feed_service::list_visits(pool, f),
        || feed_service::count_visits(pool, f),
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/feed/types",
    responses(
        (status = 200, description = "List of feed types", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn list_types(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    simple_list(
        feed_service::list_types(pool),
        feed_service::count_types(pool),
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/feed/groups",
    responses(
        (status = 200, description = "List of feed groups", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn list_groups(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    simple_list(
        feed_service::list_groups(pool),
        feed_service::count_groups(pool),
    )
    .await
}
