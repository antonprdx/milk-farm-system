use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::analytics::{
    AlertsResponse, CullingSurvivalResponse, EnergyBalanceResponse, FeedForecastResponse,
    FertilityWindowResponse, HealthIndexResponse, KpiResponse, LactationCurveResponse,
    MastitisRiskResponse, MilkTrendResponse, ProfitabilityResponse, QuarterHealthResponse,
    ReproductionForecastResponse, SeasonalResponse,
};
use crate::services::{analytics_service, predictive_service};
use crate::state::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TrendQuery {
    pub days: Option<i64>,
    pub forecast_days: Option<i64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct LactationQuery {
    pub animal_id: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct ProfitQuery {
    pub milk_price: Option<f64>,
    pub feed_price: Option<f64>,
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
        .route("/analytics/latest-milk", get(latest_milk))
        .route("/analytics/lactation-curves", get(lactation_curves))
        .route("/analytics/health-index", get(health_index))
        .route("/analytics/fertility-window", get(fertility_window))
        .route("/analytics/profitability", get(profitability))
        .route("/analytics/seasonal", get(seasonal))
        .route("/analytics/mastitis-risk", get(mastitis_risk))
        .route("/analytics/culling-survival", get(culling_survival))
        .route("/analytics/energy-balance", get(energy_balance))
        .route("/analytics/quarter-health", get(quarter_health))
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
    let days = params.days.unwrap_or(30).clamp(1, 365);
    let forecast_days = params.forecast_days.unwrap_or(14).clamp(1, 90);
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

#[utoipa::path(
    get,
    path = "/api/v1/analytics/latest-milk",
    responses(
        (status = 200, description = "Latest milk productions", body = Vec<crate::models::analytics::LatestMilkEntry>),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn latest_milk(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::analytics::LatestMilkEntry>>, AppError> {
    let data = analytics_service::latest_milk(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/lactation-curves",
    responses(
        (status = 200, description = "Lactation curves with Wood's model fit", body = Vec<LactationCurveResponse>),
        (status = 401, description = "Unauthorized")
    ),
    params(LactationQuery),
    security(("cookie_auth" = []))
)]
async fn lactation_curves(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<LactationQuery>,
) -> Result<Json<Vec<LactationCurveResponse>>, AppError> {
    let data = predictive_service::lactation_curves(&state.pool, params.animal_id).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/health-index",
    responses(
        (status = 200, description = "Health index scores", body = HealthIndexResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn health_index(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<HealthIndexResponse>, AppError> {
    let data = predictive_service::health_index(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/fertility-window",
    responses(
        (status = 200, description = "Fertility window detection", body = FertilityWindowResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn fertility_window(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<FertilityWindowResponse>, AppError> {
    let data = predictive_service::fertility_window(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/profitability",
    responses(
        (status = 200, description = "Per-cow profitability analysis", body = ProfitabilityResponse),
        (status = 401, description = "Unauthorized")
    ),
    params(ProfitQuery),
    security(("cookie_auth" = []))
)]
async fn profitability(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<ProfitQuery>,
) -> Result<Json<ProfitabilityResponse>, AppError> {
    let milk_price = params.milk_price.unwrap_or(25.0).clamp(1.0, 200.0);
    let feed_price = params.feed_price.unwrap_or(12.0).clamp(1.0, 200.0);
    let data = predictive_service::profitability(&state.pool, milk_price, feed_price).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/seasonal",
    responses(
        (status = 200, description = "Seasonal decomposition of milk production", body = SeasonalResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn seasonal(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<SeasonalResponse>, AppError> {
    let data = predictive_service::seasonal_decomposition(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/mastitis-risk",
    responses(
        (status = 200, description = "Mastitis risk assessment", body = MastitisRiskResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn mastitis_risk(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<MastitisRiskResponse>, AppError> {
    if let Some(ref ml) = state.ml {
        let data = ml.mastitis_risk(None, &state.pool).await?;
        Ok(Json(data))
    } else {
        let data = predictive_service::mastitis_risk(&state.pool).await?;
        Ok(Json(data))
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/culling-survival",
    responses(
        (status = 200, description = "Culling survival estimates", body = CullingSurvivalResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn culling_survival(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<CullingSurvivalResponse>, AppError> {
    if let Some(ref ml) = state.ml {
        let data = ml.culling_survival(None, &state.pool).await?;
        Ok(Json(data))
    } else {
        let data = predictive_service::culling_survival(&state.pool).await?;
        Ok(Json(data))
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/energy-balance",
    responses(
        (status = 200, description = "Energy balance via fat/protein ratio", body = EnergyBalanceResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn energy_balance(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<EnergyBalanceResponse>, AppError> {
    let data = predictive_service::energy_balance(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/quarter-health",
    responses(
        (status = 200, description = "Per-quarter udder health analysis", body = QuarterHealthResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn quarter_health(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<QuarterHealthResponse>, AppError> {
    let data = predictive_service::quarter_health(&state.pool).await?;
    Ok(Json(data))
}
