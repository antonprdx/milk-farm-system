use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::analytics::{
    AlertsResponse, FeedForecastResponse, KpiResponse, MilkTrendResponse,
    ReproductionForecastResponse,
};
use crate::services::analytics_service;
use crate::state::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TrendQuery {
    pub days: Option<i64>,
    pub forecast_days: Option<i64>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/analytics/kpi", get(kpi))
        .route("/analytics/alerts", get(alerts))
        .route("/analytics/milk-trend", get(milk_trend))
        .route(
            "/analytics/reproduction-forecast",
            get(reproduction_forecast),
        )
        .route("/analytics/feed-forecast", get(feed_forecast))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/kpi",
    responses(
        (status = 200, description = "KPI metrics", body = KpiResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn kpi(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::analytics::KpiResponse>, AppError> {
    let data = analytics_service::kpi(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/alerts",
    responses(
        (status = 200, description = "Alerts", body = AlertsResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn alerts(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::analytics::AlertsResponse>, AppError> {
    let data = analytics_service::alerts(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/milk-trend",
    responses(
        (status = 200, description = "Milk trend data", body = MilkTrendResponse),
        (status = 401, description = "Unauthorized")
    ),
    params(TrendQuery),
    security(("cookie_auth" = []))
)]
async fn milk_trend(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<TrendQuery>,
) -> Result<Json<crate::models::analytics::MilkTrendResponse>, AppError> {
    let days = params.days.unwrap_or(30);
    let forecast_days = params.forecast_days.unwrap_or(14);
    let data = analytics_service::milk_trend(&state.pool, days, forecast_days).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/reproduction-forecast",
    responses(
        (status = 200, description = "Reproduction forecast", body = ReproductionForecastResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn reproduction_forecast(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::analytics::ReproductionForecastResponse>, AppError> {
    let data = analytics_service::reproduction_forecast(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/feed-forecast",
    responses(
        (status = 200, description = "Feed forecast", body = FeedForecastResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn feed_forecast(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::analytics::FeedForecastResponse>, AppError> {
    let data = analytics_service::feed_forecast(&state.pool).await?;
    Ok(Json(data))
}
