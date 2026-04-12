use axum::extract::{Path, Query, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, Claims};
use crate::models::feed::{
    CreateFeedDayAmount, CreateFeedGroup, CreateFeedType, FeedFilter, FeedGroup, FeedType,
    UpdateFeedGroup, UpdateFeedType,
};
use crate::models::pagination::{paginated, simple_list};
use crate::services::feed_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/feed/day-amounts",
            get(list_day_amounts).post(create_day_amount),
        )
        .route("/feed/visits", get(list_visits))
        .route("/feed/types", get(list_types).post(create_type))
        .route("/feed/types/{id}", put(update_type).delete(delete_type))
        .route("/feed/groups", get(list_groups).post(create_group))
        .route("/feed/groups/{id}", put(update_group).delete(delete_group))
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
    post,
    path = "/api/v1/feed/day-amounts",
    request_body = CreateFeedDayAmount,
    responses(
        (status = 200, description = "Feed day amount created", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create_day_amount(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(data): Json<CreateFeedDayAmount>,
) -> Result<Json<Value>, AppError> {
    data.validate()?;
    let result = feed_service::create_day_amount(&state.pool, &data).await?;
    Ok(Json(json!({ "data": result })))
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

#[utoipa::path(
    post,
    path = "/api/v1/feed/types",
    request_body = CreateFeedType,
    responses(
        (status = 200, description = "Feed type created", body = FeedType),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create_type(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(data): Json<CreateFeedType>,
) -> Result<Json<FeedType>, AppError> {
    data.validate()?;
    let feed_type = feed_service::create_feed_type(&state.pool, &data).await?;
    Ok(Json(feed_type))
}

#[utoipa::path(
    put,
    path = "/api/v1/feed/types/{id}",
    request_body = UpdateFeedType,
    responses(
        (status = 200, description = "Feed type updated", body = FeedType),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Feed type ID")),
    security(("cookie_auth" = []))
)]
async fn update_type(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(data): Json<UpdateFeedType>,
) -> Result<Json<FeedType>, AppError> {
    data.validate()?;
    let feed_type = feed_service::update_feed_type(&state.pool, id, &data).await?;
    Ok(Json(feed_type))
}

#[utoipa::path(
    delete,
    path = "/api/v1/feed/types",
    responses(
        (status = 200, description = "Feed type deleted"),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn delete_type(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    feed_service::delete_feed_type(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Тип корма удалён" })))
}

#[utoipa::path(
    post,
    path = "/api/v1/feed/groups",
    request_body = CreateFeedGroup,
    responses(
        (status = 200, description = "Feed group created", body = FeedGroup),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create_group(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(data): Json<CreateFeedGroup>,
) -> Result<Json<FeedGroup>, AppError> {
    data.validate()?;
    let group = feed_service::create_feed_group(&state.pool, &data).await?;
    Ok(Json(group))
}

#[utoipa::path(
    put,
    path = "/api/v1/feed/groups/{id}",
    request_body = UpdateFeedGroup,
    responses(
        (status = 200, description = "Feed group updated", body = FeedGroup),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Feed group ID")),
    security(("cookie_auth" = []))
)]
async fn update_group(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(data): Json<UpdateFeedGroup>,
) -> Result<Json<FeedGroup>, AppError> {
    data.validate()?;
    let group = feed_service::update_feed_group(&state.pool, id, &data).await?;
    Ok(Json(group))
}

#[utoipa::path(
    delete,
    path = "/api/v1/feed/groups",
    responses(
        (status = 200, description = "Feed group deleted"),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn delete_group(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    feed_service::delete_feed_group(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Группа корма удалена" })))
}
