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
use crate::services::{pdf_service, reports_service};
use crate::state::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
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
        .route("/reports/export/milk/pdf", get(export_milk_pdf))
        .route(
            "/reports/export/reproduction/pdf",
            get(export_reproduction_pdf),
        )
        .route("/reports/export/feed/pdf", get(export_feed_pdf))
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

fn pdf_response(filename: &str, data: Vec<u8>) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        data,
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

fn format_date_range(filter: &ReportFilter) -> String {
    format!(
        "Период: {} - {}",
        filter.from_date.map_or("-".into(), |d| d.to_string()),
        filter.till_date.map_or("-".into(), |d| d.to_string())
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/milk-summary",
    responses(
        (status = 200, description = "Milk summary report", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn milk_summary(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Value>, AppError> {
    let s = reports_service::milk_summary(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(Json(json!({
        "total_milk": s.total_milk,
        "count_days": s.count_days,
        "avg_per_animal": s.avg_per_animal,
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/reproduction-summary",
    responses(
        (status = 200, description = "Reproduction summary report", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn reproduction_summary(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Value>, AppError> {
    let s = reports_service::reproduction_summary(&state.pool, filter.from_date, filter.till_date)
        .await?;
    Ok(Json(json!({
        "total_calvings": s.total_calvings,
        "total_inseminations": s.total_inseminations,
        "total_pregnancies": s.total_pregnancies,
        "total_heats": s.total_heats,
        "total_dry_offs": s.total_dry_offs,
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/feed-summary",
    responses(
        (status = 200, description = "Feed summary report", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn feed_summary(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Value>, AppError> {
    let s = reports_service::feed_summary(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(Json(json!({
        "total_feed_kg": s.total_feed_kg,
        "total_visits": s.total_visits,
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/export/milk",
    responses(
        (status = 200, description = "CSV file download"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn export_milk(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Response, AppError> {
    validate_date_range(&filter)?;
    let csv =
        reports_service::export_milk_csv(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(csv_response("milk_report.csv", csv))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/export/reproduction",
    responses(
        (status = 200, description = "CSV file download"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn export_reproduction(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Response, AppError> {
    validate_date_range(&filter)?;
    let csv =
        reports_service::export_reproduction_csv(&state.pool, filter.from_date, filter.till_date)
            .await?;
    Ok(csv_response("reproduction_report.csv", csv))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/export/feed",
    responses(
        (status = 200, description = "CSV file download"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn export_feed(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Response, AppError> {
    validate_date_range(&filter)?;
    let csv =
        reports_service::export_feed_csv(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(csv_response("feed_report.csv", csv))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/export/milk/pdf",
    responses(
        (status = 200, description = "PDF file download"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn export_milk_pdf(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Response, AppError> {
    validate_date_range(&filter)?;
    let rows =
        reports_service::milk_export_rows(&state.pool, filter.from_date, filter.till_date).await?;
    let sections = vec![pdf_service::TableSection {
        title: None,
        headers: vec![
            "Животное".into(),
            "Дата".into(),
            "Надой (л)".into(),
            "Средний надой".into(),
            "Средний вес".into(),
            "ИСК".into(),
        ],
        rows,
    }];
    let pdf = pdf_service::generate_pdf("Отчёт по надоям", &format_date_range(&filter), &sections)?;
    Ok(pdf_response("milk_report.pdf", pdf))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/export/reproduction/pdf",
    responses(
        (status = 200, description = "PDF file download"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn export_reproduction_pdf(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Response, AppError> {
    validate_date_range(&filter)?;
    let calvings =
        reports_service::calvings_export_rows(&state.pool, filter.from_date, filter.till_date)
            .await?;
    let inseminations =
        reports_service::inseminations_export_rows(&state.pool, filter.from_date, filter.till_date)
            .await?;
    let sections = vec![
        pdf_service::TableSection {
            title: Some("Отёлы".into()),
            headers: vec![
                "Животное".into(),
                "Дата".into(),
                "Примечания".into(),
                "Лактация".into(),
            ],
            rows: calvings,
        },
        pdf_service::TableSection {
            title: Some("Инсеминации".into()),
            headers: vec![
                "Животное".into(),
                "Дата".into(),
                "Код быка".into(),
                "Тип".into(),
            ],
            rows: inseminations,
        },
    ];
    let pdf = pdf_service::generate_pdf(
        "Отчёт по воспроизводству",
        &format_date_range(&filter),
        &sections,
    )?;
    Ok(pdf_response("reproduction_report.pdf", pdf))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/export/feed/pdf",
    responses(
        (status = 200, description = "PDF file download"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn export_feed_pdf(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Response, AppError> {
    validate_date_range(&filter)?;
    let rows =
        reports_service::feed_export_rows(&state.pool, filter.from_date, filter.till_date).await?;
    let sections = vec![pdf_service::TableSection {
        title: None,
        headers: vec![
            "Животное".into(),
            "Дата".into(),
            "Номер корма".into(),
            "Количество (кг)".into(),
        ],
        rows,
    }];
    let pdf =
        pdf_service::generate_pdf("Отчёт по кормлению", &format_date_range(&filter), &sections)?;
    Ok(pdf_response("feed_report.pdf", pdf))
}
