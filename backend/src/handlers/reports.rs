use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ReportFilter {
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/reports/milk-summary", get(milk_summary))
        .route("/reports/reproduction-summary", get(reproduction_summary))
        .route("/reports/feed-summary", get(feed_summary))
        .route("/reports/export/milk", get(export_milk))
        .route("/reports/export/reproduction", get(export_reproduction))
        .route("/reports/export/feed", get(export_feed))
}

fn csv_response(filename: &str, csv: String) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        csv,
    )
        .into_response()
}

fn validate_date_range(filter: &ReportFilter) -> Result<(), AppError> {
    match (filter.from_date, filter.till_date) {
        (Some(from), Some(till)) if (till - from).num_days() > 366 => {
            return Err(AppError::BadRequest(
                "Диапазон дат не может превышать 1 год".into(),
            ));
        }
        _ => {}
    }
    Ok(())
}

async fn milk_summary(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Value>, AppError> {
    let s = crate::services::reports_service::milk_summary(
        &state.pool,
        filter.from_date,
        filter.till_date,
    )
    .await?;
    Ok(Json(json!({
        "total_milk": s.total_milk,
        "count_days": s.count_days,
        "avg_per_animal": s.avg_per_animal,
    })))
}

async fn reproduction_summary(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Value>, AppError> {
    let s = crate::services::reports_service::reproduction_summary(
        &state.pool,
        filter.from_date,
        filter.till_date,
    )
    .await?;
    Ok(Json(json!({
        "total_calvings": s.total_calvings,
        "total_inseminations": s.total_inseminations,
        "total_pregnancies": s.total_pregnancies,
        "total_heats": s.total_heats,
        "total_dry_offs": s.total_dry_offs,
    })))
}

async fn feed_summary(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Value>, AppError> {
    let s = crate::services::reports_service::feed_summary(
        &state.pool,
        filter.from_date,
        filter.till_date,
    )
    .await?;
    Ok(Json(json!({
        "total_feed_kg": s.total_feed_kg,
        "total_visits": s.total_visits,
    })))
}

async fn export_milk(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Response, AppError> {
    validate_date_range(&filter)?;
    let csv = crate::services::reports_service::export_milk_csv(
        &state.pool,
        filter.from_date,
        filter.till_date,
    )
    .await?;
    Ok(csv_response("milk_report.csv", csv))
}

async fn export_reproduction(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Response, AppError> {
    validate_date_range(&filter)?;
    let csv = crate::services::reports_service::export_reproduction_csv(
        &state.pool,
        filter.from_date,
        filter.till_date,
    )
    .await?;
    Ok(csv_response("reproduction_report.csv", csv))
}

async fn export_feed(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Response, AppError> {
    validate_date_range(&filter)?;
    let csv = crate::services::reports_service::export_feed_csv(
        &state.pool,
        filter.from_date,
        filter.till_date,
    )
    .await?;
    Ok(csv_response("feed_report.csv", csv))
}
