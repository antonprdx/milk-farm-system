use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::reports::{
    CalendarResponse, FeedPerTypeResponse, HealthTaskResponse, HerdOverviewResponse,
    PregnancyRateResponse, RestFeedResponse, TransitionResponse, UdderHealthResponse,
};
use crate::services::{pdf_service, reports_service};
use crate::state::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct ReportFilter {
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct LactationFilter {
    pub lac_number: Option<i32>,
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
        .route("/reports/herd-overview", get(herd_overview))
        .route("/reports/rest-feed", get(rest_feed))
        .route("/reports/robot-performance", get(robot_performance))
        .route("/reports/failed-milkings", get(failed_milkings))
        .route("/reports/udder-health-worklist", get(udder_health_worklist))
        .route("/reports/udder-health-analyze", get(udder_health_analyze))
        .route(
            "/reports/milk-day-production-time",
            get(milk_day_production_time),
        )
        .route("/reports/visit-behavior", get(visit_behavior))
        .route("/reports/calendar", get(calendar))
        .route(
            "/reports/health-activity-rumination",
            get(health_activity_rumination),
        )
        .route("/reports/cow-robot-efficiency", get(cow_robot_efficiency))
        .route("/reports/lactation-analysis", get(lactation_analysis))
        .route("/reports/feed-per-type-day", get(feed_per_type_day))
        .route("/reports/feed-per-cow-day", get(feed_per_cow_day))
        .route("/reports/health-task", get(health_task))
        .route("/reports/pregnancy-rate", get(pregnancy_rate))
        .route("/reports/transition", get(transition_report))
        .route("/reports/export/{report_type}/csv", get(export_report_csv))
        .route("/reports/export/{report_type}/pdf", get(export_report_pdf))
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

#[utoipa::path(
    get,
    path = "/api/v1/reports/herd-overview",
    responses(
        (status = 200, description = "Herd overview report", body = HerdOverviewResponse),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn herd_overview(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<crate::models::reports::HerdOverviewResponse>, AppError> {
    let data =
        reports_service::herd_overview(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/rest-feed",
    responses(
        (status = 200, description = "Rest feed report", body = RestFeedResponse),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn rest_feed(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<crate::models::reports::RestFeedResponse>, AppError> {
    let data =
        reports_service::rest_feed_report(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/robot-performance",
    responses(
        (status = 200, description = "Robot performance report"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn robot_performance(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Vec<crate::models::reports::RobotPerformanceRow>>, AppError> {
    let data =
        reports_service::robot_performance(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/failed-milkings",
    responses(
        (status = 200, description = "Failed milkings report"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn failed_milkings(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Vec<crate::models::reports::FailedMilkingRow>>, AppError> {
    let data =
        reports_service::failed_milkings(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/udder-health-worklist",
    responses(
        (status = 200, description = "Udder health worklist", body = UdderHealthResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn udder_health_worklist(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::reports::UdderHealthResponse>, AppError> {
    let data = reports_service::udder_health_worklist(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/udder-health-analyze",
    responses(
        (status = 200, description = "Udder health analysis", body = UdderHealthResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn udder_health_analyze(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::reports::UdderHealthResponse>, AppError> {
    let data = reports_service::udder_health_analyze(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/milk-day-production-time",
    responses(
        (status = 200, description = "Milk day production time report"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn milk_day_production_time(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Vec<crate::models::reports::MilkDayProductionTimeRow>>, AppError> {
    let data =
        reports_service::milk_day_production_time(&state.pool, filter.from_date, filter.till_date)
            .await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/visit-behavior",
    responses(
        (status = 200, description = "Visit behavior report"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn visit_behavior(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Vec<crate::models::reports::VisitBehaviorRow>>, AppError> {
    let data =
        reports_service::visit_behavior(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/calendar",
    responses(
        (status = 200, description = "Calendar report", body = CalendarResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn calendar(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::reports::CalendarResponse>, AppError> {
    let data = reports_service::calendar(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/health-activity-rumination",
    responses(
        (status = 200, description = "Health activity and rumination report"),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn health_activity_rumination(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::reports::HealthActivityRow>>, AppError> {
    let data = reports_service::health_activity_rumination(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/cow-robot-efficiency",
    responses(
        (status = 200, description = "Cow robot efficiency report"),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn cow_robot_efficiency(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::reports::CowRobotEfficiencyRow>>, AppError> {
    let data = reports_service::cow_robot_efficiency(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/lactation-analysis",
    responses(
        (status = 200, description = "Lactation analysis report"),
        (status = 401, description = "Unauthorized")
    ),
    params(LactationFilter),
    security(("cookie_auth" = []))
)]
async fn lactation_analysis(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<LactationFilter>,
) -> Result<Json<Vec<crate::models::reports::LactationAnalysisResponse>>, AppError> {
    let data = reports_service::lactation_analysis(&state.pool, filter.lac_number).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/feed-per-type-day",
    responses(
        (status = 200, description = "Feed per type per day report", body = FeedPerTypeResponse),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn feed_per_type_day(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<crate::models::reports::FeedPerTypeResponse>, AppError> {
    let data =
        reports_service::feed_per_type_day(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/feed-per-cow-day",
    responses(
        (status = 200, description = "Feed per cow per day report"),
        (status = 401, description = "Unauthorized")
    ),
    params(ReportFilter),
    security(("cookie_auth" = []))
)]
async fn feed_per_cow_day(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Vec<crate::models::reports::FeedPerCowDayRow>>, AppError> {
    let data =
        reports_service::feed_per_cow_day(&state.pool, filter.from_date, filter.till_date).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/health-task",
    responses(
        (status = 200, description = "Health task report", body = HealthTaskResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn health_task(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::reports::HealthTaskResponse>, AppError> {
    let data = reports_service::health_task(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/pregnancy-rate",
    responses(
        (status = 200, description = "Pregnancy rate report", body = PregnancyRateResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn pregnancy_rate(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::reports::PregnancyRateResponse>, AppError> {
    let data = reports_service::pregnancy_rate_report(&state.pool).await?;
    Ok(Json(data))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/transition",
    responses(
        (status = 200, description = "Transition report", body = TransitionResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn transition_report(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<crate::models::reports::TransitionResponse>, AppError> {
    let data = reports_service::transition_report(&state.pool).await?;
    Ok(Json(data))
}

async fn get_report_json(
    report_type: &str,
    pool: &sqlx::PgPool,
    filter: &ReportFilter,
    lac_number: Option<i32>,
) -> Result<(String, Value), AppError> {
    let (title, data) = match report_type {
        "herd-overview" => (
            "Обзор стада".to_string(),
            serde_json::to_value(
                reports_service::herd_overview(pool, filter.from_date, filter.till_date).await?,
            )
            .unwrap_or_default(),
        ),
        "rest-feed" => (
            "Остаток корма".to_string(),
            serde_json::to_value(
                reports_service::rest_feed_report(pool, filter.from_date, filter.till_date).await?,
            )
            .unwrap_or_default(),
        ),
        "robot-performance" => (
            "Робот".to_string(),
            serde_json::to_value(
                reports_service::robot_performance(pool, filter.from_date, filter.till_date)
                    .await?,
            )
            .unwrap_or_default(),
        ),
        "failed-milkings" => (
            "Неудачные доения".to_string(),
            serde_json::to_value(
                reports_service::failed_milkings(pool, filter.from_date, filter.till_date).await?,
            )
            .unwrap_or_default(),
        ),
        "udder-health-worklist" => (
            "Здоровье вымени (R12)".to_string(),
            serde_json::to_value(reports_service::udder_health_worklist(pool).await?)
                .unwrap_or_default(),
        ),
        "udder-health-analyze" => (
            "Анализ вымени (R23)".to_string(),
            serde_json::to_value(reports_service::udder_health_analyze(pool).await?)
                .unwrap_or_default(),
        ),
        "milk-day-production-time" => (
            "Надой по времени".to_string(),
            serde_json::to_value(
                reports_service::milk_day_production_time(pool, filter.from_date, filter.till_date)
                    .await?,
            )
            .unwrap_or_default(),
        ),
        "visit-behavior" => (
            "Визиты".to_string(),
            serde_json::to_value(
                reports_service::visit_behavior(pool, filter.from_date, filter.till_date).await?,
            )
            .unwrap_or_default(),
        ),
        "calendar" => (
            "Календарь".to_string(),
            serde_json::to_value(reports_service::calendar(pool).await?).unwrap_or_default(),
        ),
        "health-activity-rumination" => (
            "Активность/жвачка".to_string(),
            serde_json::to_value(reports_service::health_activity_rumination(pool).await?)
                .unwrap_or_default(),
        ),
        "cow-robot-efficiency" => (
            "Эффективность".to_string(),
            serde_json::to_value(reports_service::cow_robot_efficiency(pool).await?)
                .unwrap_or_default(),
        ),
        "lactation-analysis" => (
            "Лактация".to_string(),
            serde_json::to_value(reports_service::lactation_analysis(pool, lac_number).await?)
                .unwrap_or_default(),
        ),
        "feed-per-type-day" => (
            "Корм по типам".to_string(),
            serde_json::to_value(
                reports_service::feed_per_type_day(pool, filter.from_date, filter.till_date)
                    .await?,
            )
            .unwrap_or_default(),
        ),
        "feed-per-cow-day" => (
            "Корм на корову".to_string(),
            serde_json::to_value(
                reports_service::feed_per_cow_day(pool, filter.from_date, filter.till_date).await?,
            )
            .unwrap_or_default(),
        ),
        "health-task" => (
            "Здоровье (sick chance)".to_string(),
            serde_json::to_value(reports_service::health_task(pool).await?).unwrap_or_default(),
        ),
        "pregnancy-rate" => (
            "Коэфф. стельности".to_string(),
            serde_json::to_value(reports_service::pregnancy_rate_report(pool).await?)
                .unwrap_or_default(),
        ),
        "transition" => (
            "Транзитный период".to_string(),
            serde_json::to_value(reports_service::transition_report(pool).await?)
                .unwrap_or_default(),
        ),
        _ => return Err(AppError::BadRequest("Неизвестный тип отчёта".into())),
    };
    Ok((title, data))
}

fn flatten_to_rows(val: &Value) -> Vec<Vec<String>> {
    fn fmt_val(v: &Value) -> String {
        match v {
            Value::Null => String::new(),
            Value::Bool(b) => if *b { "Да" } else { "Нет" }.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(arr) => arr.iter().map(fmt_val).collect::<Vec<_>>().join(", "),
            _ => String::new(),
        }
    }

    let items: Vec<&Value> = if let Some(arr) = val.as_array() {
        arr.iter().collect()
    } else if val.is_object() {
        vec![val]
    } else {
        return vec![];
    };

    let mut result = Vec::new();
    for obj in &items {
        if let Some(map) = obj.as_object() {
            let row: Vec<String> = map.values().map(fmt_val).collect();
            if !result.is_empty() || !row.is_empty() {
                result.push(row);
            }
        }
    }
    result
}

fn extract_headers(val: &Value) -> Vec<String> {
    let obj = if let Some(arr) = val.as_array() {
        arr.first()
    } else {
        Some(val)
    };
    obj.and_then(|v| v.as_object())
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default()
}

fn json_to_csv(headers: &[String], rows: &[Vec<String>]) -> String {
    let escape = |s: &str| -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    };
    let mut lines = vec![
        headers
            .iter()
            .map(|h| escape(h))
            .collect::<Vec<_>>()
            .join(","),
    ];
    for row in rows {
        lines.push(row.iter().map(|c| escape(c)).collect::<Vec<_>>().join(","));
    }
    lines.join("\n")
}

fn data_to_table_sections(val: &Value) -> Vec<pdf_service::TableSection> {
    let mut sections = Vec::new();

    if let Some(map) = val.as_object() {
        for (key, inner) in map {
            if let Some(_arr) = inner.as_array() {
                let headers = extract_headers(inner);
                let rows = flatten_to_rows(inner);
                if !headers.is_empty() {
                    sections.push(pdf_service::TableSection {
                        title: Some(key.clone()),
                        headers,
                        rows,
                    });
                }
            } else if inner.is_object()
                && inner
                    .as_object()
                    .is_some_and(|m| !m.contains_key("period") && !m.contains_key("rows"))
            {
                continue;
            }
        }
        if sections.is_empty() {
            let headers = extract_headers(val);
            let rows = flatten_to_rows(val);
            if !headers.is_empty() {
                sections.push(pdf_service::TableSection {
                    title: None,
                    headers,
                    rows,
                });
            }
        }
    } else if val.is_array() {
        let headers = extract_headers(val);
        let rows = flatten_to_rows(val);
        if !headers.is_empty() {
            sections.push(pdf_service::TableSection {
                title: None,
                headers,
                rows,
            });
        }
    }

    sections
}

#[derive(Debug, Deserialize)]
pub struct ExportFilter {
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub lac_number: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/export/{report_type}/csv",
    responses(
        (status = 200, description = "CSV file download"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Unknown report type")
    ),
    params(("report_type" = String, Path, description = "Report type identifier")),
    security(("cookie_auth" = []))
)]
async fn export_report_csv(
    _claims: Claims,
    State(state): State<AppState>,
    Path(report_type): Path<String>,
    Query(filter): Query<ExportFilter>,
) -> Result<Response, AppError> {
    let rf = ReportFilter {
        from_date: filter.from_date,
        till_date: filter.till_date,
    };
    let (title, data) = get_report_json(&report_type, &state.pool, &rf, filter.lac_number).await?;
    let headers = extract_headers(&data);
    let rows = flatten_to_rows(&data);
    let csv = json_to_csv(&headers, &rows);
    let filename = format!("{}.csv", title.replace(' ', "_"));
    Ok(csv_response(&filename, csv))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/export/{report_type}/pdf",
    responses(
        (status = 200, description = "PDF file download"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Unknown report type")
    ),
    params(("report_type" = String, Path, description = "Report type identifier")),
    security(("cookie_auth" = []))
)]
async fn export_report_pdf(
    _claims: Claims,
    State(state): State<AppState>,
    Path(report_type): Path<String>,
    Query(filter): Query<ExportFilter>,
) -> Result<Response, AppError> {
    let rf = ReportFilter {
        from_date: filter.from_date,
        till_date: filter.till_date,
    };
    let (title, data) = get_report_json(&report_type, &state.pool, &rf, filter.lac_number).await?;
    let sections = data_to_table_sections(&data);
    let subtitle = format!(
        "Период: {} - {}",
        rf.from_date.map_or("-".into(), |d| d.to_string()),
        rf.till_date.map_or("-".into(), |d| d.to_string())
    );
    let pdf = pdf_service::generate_pdf(&title, &subtitle, &sections)?;
    let filename = format!("{}.pdf", title.replace(' ', "_"));
    Ok(pdf_response(&filename, pdf))
}
