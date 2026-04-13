mod common;

use axum::http::StatusCode;
use milk_farm_backend::create_app;
use serde_json::Value;
use tower::ServiceExt;

use common::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_herd_overview_requires_date_range(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/herd-overview", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_herd_overview_with_dates(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/herd-overview?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["period"].is_array());
    assert!(body["avg_cow_count"].is_number());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_rest_feed_requires_date_range(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/rest-feed", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_rest_feed_with_dates(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/rest-feed?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["rows"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_robot_performance_requires_date_range(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/robot-performance", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_robot_performance_with_dates(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/robot-performance?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body.is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_failed_milkings_requires_date_range(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/failed-milkings", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_failed_milkings_with_dates(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/failed-milkings?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body.is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_udder_health_worklist(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/udder-health-worklist", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["rows"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_udder_health_analyze(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/udder-health-analyze", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["rows"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_milk_day_production_time_requires_date_range(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/milk-day-production-time", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_milk_day_production_time_with_dates(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/milk-day-production-time?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body.is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_visit_behavior_requires_date_range(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/visit-behavior", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_visit_behavior_with_dates(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/visit-behavior?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body.is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_calendar(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/calendar", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["expected_calvings"].is_array());
    assert!(body["expected_dry_offs"].is_array());
    assert!(body["expected_heats"].is_array());
    assert!(body["pregnancy_checks"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_health_activity_rumination(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/health-activity-rumination", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body.is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cow_robot_efficiency(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/cow-robot-efficiency", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body.is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lactation_analysis(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/lactation-analysis", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body.is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_feed_per_type_day_requires_date_range(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/feed-per-type-day", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_feed_per_type_day_with_dates(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/feed-per-type-day?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["rows"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_feed_per_cow_day_requires_date_range(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/feed-per-cow-day", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_feed_per_cow_day_with_dates(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/feed-per-cow-day?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body.is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_health_task(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/health-task", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["rows"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_pregnancy_rate(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/pregnancy-rate", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["periods"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_transition_report(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/reports/transition", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["rows"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_export_milk_csv(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/export/milk?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_string(resp.into_body()).await;
    assert!(body.contains("Дата") || body.contains("date") || !body.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_export_milk_pdf(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/export/milk/pdf?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(ct.contains("pdf"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_export_reproduction_csv(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/export/reproduction?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_export_feed_csv(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/export/feed?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_generic_export_csv(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/export/herd-overview/csv?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_generic_export_pdf(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/export/herd-overview/pdf?from_date=2025-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(ct.contains("pdf"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_generic_export_no_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder()
        .uri("/api/v1/reports/export/herd-overview/csv?from_date=2025-01-01&till_date=2025-06-01")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_date_range_too_large(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/reports/herd-overview?from_date=2023-01-01&till_date=2025-06-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
