mod common;

use axum::http::StatusCode;
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_status_requires_admin(pool: sqlx::PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/lely/status", &user_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_status_requires_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder()
        .uri("/api/v1/lely/status")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_status_returns_data(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/lely/status", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["data"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_get_config(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/lely/config", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["enabled"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_get_config_requires_admin(pool: sqlx::PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/lely/config", &user_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_update_config(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "PUT",
        "/api/v1/lely/config",
        &admin_token(),
        json!({ "sync_interval_secs": 600 }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["message"], "Настройки Lely сохранены");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_update_config_validation_bad_url(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "PUT",
        "/api/v1/lely/config",
        &admin_token(),
        json!({ "base_url": "ftp://invalid.example.com" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_update_config_validation_bad_interval(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "PUT",
        "/api/v1/lely/config",
        &admin_token(),
        json!({ "sync_interval_secs": 10 }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_trigger_sync_disabled(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/lely/sync",
        &admin_token(),
        json!({}),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_test_connection_requires_admin(pool: sqlx::PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/lely/test-connection",
        &user_token(),
        json!({}),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_lely_test_connection_missing_fields(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/lely/test-connection",
        &admin_token(),
        json!({ "base_url": "https://example.com" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
