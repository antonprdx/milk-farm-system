mod common;

use axum::http::StatusCode;
use milk_farm_backend::create_app;
use serde_json::Value;
use tower::ServiceExt;

use common::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_analytics_kpi_requires_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder()
        .uri("/api/v1/analytics/kpi")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_analytics_kpi_returns_data(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/analytics/kpi", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["avg_calving_interval_days"].is_number() || body["avg_calving_interval_days"].is_null());
    assert!(body["avg_milk_by_lactation"].is_array());
    assert!(body["culling_risk"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_analytics_alerts_returns_data(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/analytics/alerts", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["alerts"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_analytics_milk_trend_returns_data(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/analytics/milk-trend", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["daily"].is_array());
    assert!(body["forecast"].is_array());
    assert!(body["trend_direction"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_analytics_milk_trend_with_params(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/analytics/milk-trend?days=7&forecast_days=3",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_analytics_reproduction_forecast_returns_data(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/v1/analytics/reproduction-forecast",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["expected_calvings"].is_array());
    assert!(body["expected_heats"].is_array());
    assert!(body["dry_off_recommendations"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_analytics_feed_forecast_returns_data(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/analytics/feed-forecast", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["predicted_next_week_kg"].is_number());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_analytics_latest_milk_returns_data(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/analytics/latest-milk", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body.is_array());
}
