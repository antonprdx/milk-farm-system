mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

fn make_app(pool: sqlx::PgPool) -> axum::Router {
    create_app(app_state(pool))
}

#[sqlx::test(migrations = "./migrations")]
async fn test_health_with_db(pool: sqlx::PgPool) {
    let app = make_app(pool);
    let req = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = read_body_string(resp.into_body()).await;
    assert!(body.contains("\"db\":\"ok\""));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_login_invalid_json(pool: sqlx::PgPool) {
    let app = make_app(pool);
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from("{invalid}"))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_login_empty_fields(pool: sqlx::PgPool) {
    let app = make_app(pool);
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"username":"","password":""}"#))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert!(resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_animal_missing_required(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/animals")
        .header("Authorization", format!("Bearer {}", admin_token()))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"gender":"female"}"#))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_nonexistent_animal(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);
    let req = auth_request_with_body(
        "PUT",
        "/api/v1/animals/99999",
        &admin_token(),
        json!({"name": "Ghost"}),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert!(
        resp.status() == StatusCode::NOT_FOUND
            || resp.status() == StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_nonexistent_milk(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);
    let req = auth_request(
        "DELETE",
        "/api/v1/milk/day-productions/99999",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_register_duplicate_username(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool.clone());

    let body = json!({"username": "dupuser", "password": "password123"});
    let req = auth_request_with_body("POST", "/api/v1/auth/register", &admin_token(), &body);
    let resp = app.oneshot(req).await.unwrap();
    assert!(resp.status().is_success());

    let app2 = make_app(pool);
    let req2 = auth_request_with_body("POST", "/api/v1/auth/register", &admin_token(), &body);
    let resp2 = app2.oneshot(req2).await.unwrap();
    assert!(resp2.status() == StatusCode::CONFLICT || resp2.status() == StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_animal_filter_by_gender(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    seed_animal(&pool).await;
    seed_male_animal(&pool).await;

    let app = make_app(pool);
    let req = auth_request("GET", "/api/v1/animals?gender=female", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = read_body_json(resp.into_body()).await;
    let animals = body["data"].as_array().unwrap();
    assert!(animals.iter().all(|a| a["gender"] == "female"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_animal_filter_by_active(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    seed_animal(&pool).await;

    let app = make_app(pool);
    let req = auth_request("GET", "/api/v1/animals?active=false", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = read_body_json(resp.into_body()).await;
    let animals = body["data"].as_array().unwrap();
    assert!(animals.iter().all(|a| a["active"] == false));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_unauthorized_no_token(pool: sqlx::PgPool) {
    let app = make_app(pool);
    let req = Request::builder()
        .uri("/api/v1/animals")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_unauthorized_invalid_token(pool: sqlx::PgPool) {
    let app = make_app(pool);
    let req = Request::builder()
        .uri("/api/v1/animals")
        .header("Authorization", "Bearer invalidtoken123")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_delete_contact(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    seed_test_user(&pool).await;
    let app = make_app(pool);
    let req = auth_request("DELETE", "/api/v1/contacts/1", &user_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_csv_export_date_range_validation(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);
    let req = auth_request(
        "GET",
        "/api/v1/reports/export/milk?from_date=2020-01-01&till_date=2022-01-01",
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_logout_clears_cookies(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool.clone());

    let login_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from(
            r#"{"username":"admin","password":"admin12345"}"#,
        ))
        .unwrap();
    let login_resp = app.oneshot(login_req).await.unwrap();
    assert!(login_resp.status().is_success());
    let cookies: Vec<_> = login_resp.headers().get_all("set-cookie").iter().collect();
    assert!(!cookies.is_empty());

    let app2 = make_app(pool);
    let logout_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/logout")
        .header("Cookie", "token=sometoken")
        .body(Body::empty())
        .unwrap();
    let logout_resp = app2.oneshot(logout_req).await.unwrap();
    assert!(logout_resp.status().is_success());

    let set_cookies: Vec<_> = logout_resp.headers().get_all("set-cookie").iter().collect();
    assert!(!set_cookies.is_empty());
    for c in set_cookies {
        let val = c.to_str().unwrap();
        assert!(val.contains("Max-Age=0"), "Cookie should clear: {}", val);
    }
}
