use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::analytics::{
    AnimalSummaryResponse, CowClusterResponse, CullingSurvivalResponse,
    DryOffOptimizerResponse, EnergyBalanceResponse, EquipmentAnomalyResponse,
    EnsembleForecastResponse, EstrusResponse, FeedEfficiencyResponse, FeedForecastResponse,
    FeedRecommendationResponse, FertilityWindowResponse, HealthIndexResponse,
    HealthTimelineResponse, KetosisWarningResponse, KpiResponse, LactationCurveResponse,
    LifetimeValueResponse, MastitisRiskResponse, MilkForecastDataResponse, MilkTrendResponse,
    QuarterHealthResponse, ReproductionForecastResponse, SeasonalResponse,
    TimeSeriesComparisonResponse,
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
pub struct ForecastQuery {
    pub animal_id: i32,
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct ClusterQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct AnimalQuery {
    pub animal_id: Option<i32>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/analytics/kpi", get(kpi))
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
        .route("/analytics/seasonal", get(seasonal))
        .route("/analytics/mastitis-risk", get(mastitis_risk))
        .route("/analytics/culling-survival", get(culling_survival))
        .route("/analytics/energy-balance", get(energy_balance))
        .route("/analytics/quarter-health", get(quarter_health))
        .route("/analytics/milk-forecast", get(milk_forecast))
        .route("/analytics/cow-clusters", get(cow_clusters))
        .route("/analytics/estrus", get(estrus_detection))
        .route("/analytics/equipment-anomaly", get(equipment_anomaly))
        .route("/analytics/feed-recommendation", get(feed_recommendation))
        .route("/analytics/ketosis-warning", get(ketosis_warning))
        .route("/analytics/feed-efficiency", get(feed_efficiency))
        .route("/analytics/dry-off-optimizer", get(dry_off_optimizer))
        .route("/analytics/lifetime-value", get(lifetime_value))
        .route("/analytics/animal-summary", get(animal_summary))
        .route("/analytics/health-timeline", get(health_timeline))
        .route("/analytics/time-series-comparison", get(time_series_comparison))
        .route("/analytics/ensemble-forecast", get(ensemble_forecast))
        .route("/analytics/dashboard", get(dashboard))
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct DashboardQuery {
    pub trend_days: Option<i64>,
    pub forecast_days: Option<i64>,
}

async fn dashboard(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<DashboardQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let trend_days = params.trend_days.unwrap_or(30).clamp(1, 365);
    let forecast_days = params.forecast_days.unwrap_or(14).clamp(1, 90);

    if let Some(ref cache) = state.ml_cache {
        if let Some(cached) = cache.get::<serde_json::Value>("dashboard").await {
            return Ok(Json(cached));
        }
    }

    let (kpi_res, trend_res, repro_res, feed_res, milk_res, vet_res, withdrawals_res) = tokio::try_join!(
        analytics_service::kpi(&state.pool),
        analytics_service::milk_trend(&state.pool, trend_days, forecast_days),
        analytics_service::reproduction_forecast(&state.pool),
        analytics_service::feed_forecast(&state.pool),
        analytics_service::latest_milk(&state.pool),
        crate::services::vet_service::upcoming_follow_ups(&state.pool, 7),
        crate::services::vet_service::active_withdrawals(&state.pool),
    )?;

    let result = serde_json::json!({
        "kpi": kpi_res,
        "trend": trend_res,
        "reproduction": repro_res,
        "feed": feed_res,
        "latest_milk": milk_res,
        "vet_follow_ups": vet_res,
        "active_withdrawals": withdrawals_res,
    });

    if let Some(ref cache) = state.ml_cache {
        cache.set("dashboard", &result).await;
    }

    Ok(Json(result))
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
    Query(params): Query<AnimalQuery>,
) -> Result<Json<MastitisRiskResponse>, AppError> {
    if let Some(ref ml) = state.ml {
        let data = ml.mastitis_risk(params.animal_id, &state.pool).await?;
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
    Query(params): Query<AnimalQuery>,
) -> Result<Json<CullingSurvivalResponse>, AppError> {
    if let Some(ref ml) = state.ml {
        let data = ml.culling_survival(params.animal_id, &state.pool).await?;
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

#[utoipa::path(
    get,
    path = "/api/v1/analytics/milk-forecast",
    responses(
        (status = 200, description = "30-day milk production forecast", body = MilkForecastDataResponse),
        (status = 401, description = "Unauthorized")
    ),
    params(ForecastQuery),
    security(("cookie_auth" = []))
)]
async fn milk_forecast(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<ForecastQuery>,
) -> Result<Json<MilkForecastDataResponse>, AppError> {
    let days = params.days.unwrap_or(30).clamp(7, 90);
    match &state.ml {
        Some(ml) => {
            let data = ml.milk_forecast(params.animal_id, days).await?;
            Ok(Json(data))
        }
        None => Err(AppError::Internal(anyhow::anyhow!(
            "ML service unavailable"
        ))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/ensemble-forecast",
    responses(
        (status = 200, description = "Ensemble forecast combining ML and Rust models", body = EnsembleForecastResponse),
        (status = 401, description = "Unauthorized")
    ),
    params(ForecastQuery),
    security(("cookie_auth" = []))
)]
async fn ensemble_forecast(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<ForecastQuery>,
) -> Result<Json<EnsembleForecastResponse>, AppError> {
    let days = params.days.unwrap_or(30).clamp(7, 90);
    let pool = &state.pool;

    let ts = analytics_service::time_series_comparison(pool, params.animal_id, 90, days as i64).await?;

    let rust_model = ts.models.iter().find(|m| m.model_name == ts.best_model);
    let rust_mape = rust_model.map(|m| m.mape).unwrap_or(f64::MAX);
    let rust_forecast: std::collections::HashMap<i32, f64> = rust_model
        .map(|m| m.forecast.iter().enumerate().map(|(i, p)| ((i + 1) as i32, p.value)).collect())
        .unwrap_or_default();

    let rust_weight = if rust_mape > 0.0 { 1.0 / (rust_mape + 1.0) } else { 0.0 };

    let ml_result = match &state.ml {
        Some(ml) => ml.milk_forecast(params.animal_id, days).await.ok(),
        None => None,
    };

    let nbeats_result = match &state.ml {
        Some(ml) => ml.advanced_forecast(params.animal_id, days).await.ok(),
        None => None,
    };

    let nbeats_forecast: std::collections::HashMap<i32, f64> = nbeats_result
        .as_ref()
        .and_then(|r| r.nbeats.as_ref())
        .map(|b| b.forecast.iter().enumerate().map(|(i, p)| ((i + 1) as i32, p.value)).collect())
        .unwrap_or_default();

    let ml_mape = 8.0_f64;
    let nbeats_mape = 10.0_f64;
    let ml_weight = if ml_result.is_some() { 1.0 / (ml_mape + 1.0) } else { 0.0 };
    let nbeats_w = if !nbeats_forecast.is_empty() { 1.0 / (nbeats_mape + 1.0) } else { 0.0 };

    let total = ml_weight + rust_weight + nbeats_w;

    if let Some(data) = ml_result {
        let shap = data.shap_explanation.clone();
        let forecast: Vec<_> = data
            .forecast
            .iter()
            .map(|d| {
                let rust_val = rust_forecast.get(&d.day_offset).copied();
                let nb_val = nbeats_forecast.get(&d.day_offset).copied();
                let mw = ml_weight / total;
                let rw = rust_weight / total;
                let nw = nbeats_w / total;

                let (pred, ml_c, rust_c, nb_c) = match (rust_val, nb_val) {
                    (Some(rv), Some(nv)) => (d.predicted_milk * mw + rv * rw + nv * nw, mw, rw, nw),
                    (Some(rv), None) => {
                        let t = ml_weight + rust_weight;
                        let m = ml_weight / t;
                        let r = rust_weight / t;
                        (d.predicted_milk * m + rv * r, m, r, 0.0)
                    }
                    (None, Some(nv)) => {
                        let t = ml_weight + nbeats_w;
                        let m = ml_weight / t;
                        let n = nbeats_w / t;
                        (d.predicted_milk * m + nv * n, m, 0.0, n)
                    }
                    (None, None) => (d.predicted_milk, 1.0, 0.0, 0.0),
                };

                crate::models::analytics::EnsembleForecastDay {
                    day_offset: d.day_offset,
                    predicted_milk: pred,
                    lower_bound: d.lower_bound.min(pred * 0.9),
                    upper_bound: d.upper_bound.max(pred * 1.1),
                    ml_contribution: ml_c,
                    rust_contribution: rust_c,
                    nbeats_contribution: nb_c,
                }
            })
            .collect();

        let result = EnsembleForecastResponse {
            animal_id: params.animal_id,
            animal_name: data.animal_name,
            current_daily_avg: data.current_daily_avg,
            forecast,
            ml_model_version: data.model_version,
            rust_best_model: ts.best_model.clone(),
            ml_weight: if total > 0.0 { ml_weight / total } else { 0.0 },
            rust_weight: if total > 0.0 { rust_weight / total } else { 0.0 },
            nbeats_weight: if total > 0.0 { nbeats_w / total } else { 0.0 },
            rust_mape,
            shap_explanation: shap,
        };
        return Ok(Json(result));
    }

    let forecast: Vec<_> = rust_forecast
        .into_iter()
        .map(|(offset, val)| crate::models::analytics::EnsembleForecastDay {
            day_offset: offset,
            predicted_milk: val,
            lower_bound: val * 0.9,
            upper_bound: val * 1.1,
            ml_contribution: 0.0,
            rust_contribution: 1.0,
            nbeats_contribution: 0.0,
        })
        .collect();

    Ok(Json(EnsembleForecastResponse {
        animal_id: params.animal_id,
        animal_name: ts.animal_name,
        current_daily_avg: None,
        forecast,
        ml_model_version: "unavailable".to_string(),
        rust_best_model: ts.best_model,
        ml_weight: 0.0,
        rust_weight: 1.0,
        nbeats_weight: 0.0,
        rust_mape,
        shap_explanation: None,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/analytics/cow-clusters",
    responses(
        (status = 200, description = "Cow clustering analysis", body = CowClusterResponse),
        (status = 401, description = "Unauthorized")
    ),
    params(ClusterQuery),
    security(("cookie_auth" = []))
)]
async fn cow_clusters(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<ClusterQuery>,
) -> Result<Json<CowClusterResponse>, AppError> {
    let days = params.days.unwrap_or(90).clamp(30, 365);
    match &state.ml {
        Some(ml) => {
            let data = ml.cow_clusters(days).await?;
            Ok(Json(data))
        }
        None => Err(AppError::Internal(anyhow::anyhow!(
            "ML service unavailable"
        ))),
    }
}

async fn estrus_detection(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<AnimalQuery>,
) -> Result<Json<EstrusResponse>, AppError> {
    match &state.ml {
        Some(ml) => {
            let data = ml.estrus_detection(params.animal_id, &state.pool).await?;
            Ok(Json(data))
        }
        None => {
            let data = predictive_service::estrus_detection(&state.pool).await?;
            Ok(Json(data))
        }
    }
}

async fn equipment_anomaly(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<AnimalQuery>,
) -> Result<Json<EquipmentAnomalyResponse>, AppError> {
    match &state.ml {
        Some(ml) => {
            let data = ml.equipment_anomaly(params.animal_id, &state.pool).await?;
            Ok(Json(data))
        }
        None => {
            let data = predictive_service::equipment_anomaly(&state.pool).await?;
            Ok(Json(data))
        }
    }
}

async fn feed_recommendation(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<AnimalQuery>,
) -> Result<Json<FeedRecommendationResponse>, AppError> {
    match &state.ml {
        Some(ml) => {
            let data = ml.feed_recommendation(params.animal_id, &state.pool).await?;
            Ok(Json(data))
        }
        None => {
            let data = predictive_service::feed_recommendation(&state.pool).await?;
            Ok(Json(data))
        }
    }
}

async fn ketosis_warning(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<AnimalQuery>,
) -> Result<Json<KetosisWarningResponse>, AppError> {
    match &state.ml {
        Some(ml) => {
            let data = ml.ketosis_warning(params.animal_id, &state.pool).await?;
            Ok(Json(data))
        }
        None => {
            let data = predictive_service::ketosis_warning(&state.pool).await?;
            Ok(Json(data))
        }
    }
}

async fn feed_efficiency(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<FeedEfficiencyResponse>, AppError> {
    let data = predictive_service::feed_efficiency(&state.pool).await?;
    Ok(Json(data))
}

async fn dry_off_optimizer(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<DryOffOptimizerResponse>, AppError> {
    let data = predictive_service::dry_off_optimizer(&state.pool).await?;
    Ok(Json(data))
}

async fn lifetime_value(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<LifetimeValueResponse>, AppError> {
    let data = predictive_service::lifetime_value(&state.pool).await?;
    Ok(Json(data))
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct AnimalSummaryQuery {
    pub animal_id: i32,
}

async fn animal_summary(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<AnimalSummaryQuery>,
) -> Result<Json<AnimalSummaryResponse>, AppError> {
    let data = predictive_service::animal_summary(&state.pool, params.animal_id).await?;
    Ok(Json(data))
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct HealthTimelineQuery {
    pub animal_id: i32,
    pub days: Option<i64>,
}

async fn health_timeline(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<HealthTimelineQuery>,
) -> Result<Json<HealthTimelineResponse>, AppError> {
    let days = params.days.unwrap_or(90).clamp(7, 365);
    let data = predictive_service::health_timeline(&state.pool, params.animal_id, days).await?;
    Ok(Json(data))
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TimeSeriesComparisonQuery {
    pub animal_id: i32,
    pub days: Option<i64>,
    pub forecast_days: Option<i64>,
}

async fn time_series_comparison(
    _claims: Claims,
    State(state): State<AppState>,
    Query(params): Query<TimeSeriesComparisonQuery>,
) -> Result<Json<TimeSeriesComparisonResponse>, AppError> {
    let days = params.days.unwrap_or(90).clamp(30, 365);
    let forecast_days = params.forecast_days.unwrap_or(14).clamp(7, 90);
    let data = analytics_service::time_series_comparison(&state.pool, params.animal_id, days, forecast_days).await?;
    Ok(Json(data))
}
