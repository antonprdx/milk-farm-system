mod common;

use axum::body::Body;
use axum::http::StatusCode;
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

async fn create_test_animal(app: &axum::Router) -> i64 {
    let req = auth_request_with_body(
        "POST",
        "/api/animals",
        &admin_token(),
        json!({
            "gender": "female",
            "birth_date": "2020-01-01"
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn test_milk_summary(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/reports/milk-summary", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["total_milk"].is_number());
    assert!(body["count_days"].is_number());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_milk_summary_with_date_filter(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request(
        "GET",
        "/api/reports/milk-summary?from_date=2025-01-01&till_date=2025-12-31",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_reproduction_summary(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/reports/reproduction-summary", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = http_body_util::BodyExt::collect(resp.into_body())
        .await
        .unwrap()
        .to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert_eq!(
        status,
        StatusCode::OK,
        "status={}, body={}",
        status,
        body_str
    );
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body["total_calvings"].is_number());
    assert!(body["total_inseminations"].is_number());
    assert!(body["total_pregnancies"].is_number());
    assert!(body["total_heats"].is_number());
    assert!(body["total_dry_offs"].is_number());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_feed_summary(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/reports/feed-summary", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["total_feed_kg"].is_number());
    assert!(body["total_visits"].is_number());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_reports_require_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder()
        .uri("/api/reports/milk-summary")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_settings_list_users_requires_admin(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/settings/users", &user_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_settings_list_users_admin(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/settings/users", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_settings_create_user(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));

    let req = auth_request_with_body(
        "POST",
        "/api/settings/users",
        &admin_token(),
        json!({
            "username": "newuser",
            "password": "secure123"
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let list_req = auth_request("GET", "/api/settings/users", &admin_token());
    let resp2 = app.oneshot(list_req).await.unwrap();
    let body: Value = read_body_json(resp2.into_body()).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_settings_change_password(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));

    let req = auth_request_with_body(
        "POST",
        "/api/settings/password",
        &admin_token(),
        json!({
            "old_password": "admin12345",
            "new_password": "newadmin12345"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_settings_change_password_wrong_old(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = create_app(app_state(pool));

    let req = auth_request_with_body(
        "POST",
        "/api/settings/password",
        &admin_token(),
        json!({
            "old_password": "wrongpassword",
            "new_password": "newadmin12345"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_bulk_tank_list(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/bulk-tank", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_bulk_tank_create(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/bulk-tank",
        &admin_token(),
        json!({
            "date": "2025-01-15",
            "fat": 3.8,
            "protein": 3.2,
            "lactose": 4.6,
            "scc": 150
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["fat"], 3.8);
}
