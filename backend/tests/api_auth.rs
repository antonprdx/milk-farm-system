mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;
use milk_farm_backend::create_app;

use common::*;

fn make_app(pool: sqlx::PgPool) -> axum::Router {
    create_app(app_state(pool))
}

#[sqlx::test(migrations = "./migrations")]
async fn test_health_no_auth(pool: sqlx::PgPool) {
    let app = make_app(pool);
    let req = Request::builder()
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_string(resp.into_body()).await;
    assert!(body.contains("ok"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_login_valid(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;

    let hash: (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE username = 'admin'")
        .fetch_one(&pool).await.unwrap();

    let app = make_app(pool);
    let req = auth_request_with_body("POST", "/api/auth/login", "", json!({
        "username": "admin",
        "password": "admin12345"
    }));
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(status, StatusCode::OK, "body: {:?}, verify_ok: {}", body, bcrypt::verify("admin12345", &hash.0).unwrap());
    assert_eq!(body["username"], "admin");
    assert_eq!(body["role"], "admin");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_login_invalid_password(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);
    let req = auth_request_with_body("POST", "/api/auth/login", "", json!({
        "username": "admin",
        "password": "wrongpassword"
    }));
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_login_nonexistent_user(pool: sqlx::PgPool) {
    let app = make_app(pool);
    let req = auth_request_with_body("POST", "/api/auth/login", "", json!({
        "username": "ghost",
        "password": "whatever12"
    }));
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_register_requires_admin(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);
    let token = user_token();
    let req = auth_request_with_body("POST", "/api/auth/register", &token, json!({
        "username": "newuser",
        "password": "password123"
    }));
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_register_as_admin(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);
    let token = admin_token();
    let req = auth_request_with_body("POST", "/api/auth/register", &token, json!({
        "username": "newuser",
        "password": "password123"
    }));
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_stats_requires_auth(pool: sqlx::PgPool) {
    let app = make_app(pool);
    let req = Request::builder()
        .uri("/api/stats")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_stats_returns_data(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);
    let req = auth_request("GET", "/api/stats", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["total_animals"].is_number());
    assert!(body["milk_today"].is_number());
}
