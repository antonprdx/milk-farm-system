mod common;

use axum::body::Body;
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

async fn create_test_animal(app: &axum::Router) -> i64 {
    let req = auth_request_with_body(
        "POST",
        "/api/v1/animals",
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
async fn test_list_productions_empty(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/milk/day-productions", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_production(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;

    let req = auth_request_with_body(
        "POST",
        "/api/v1/milk/day-productions",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "date": "2025-01-15",
            "milk_amount": 25.5
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["milk_amount"], 25.5);
    assert_eq!(body["data"]["animal_id"], animal_id);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_get_production_by_id(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;

    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/milk/day-productions",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "date": "2025-01-15"
        }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let id = read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap();

    let get_req = auth_request(
        "GET",
        &format!("/api/v1/milk/day-productions/{}", id),
        &admin_token(),
    );
    let resp2 = app.oneshot(get_req).await.unwrap();
    let body: Value = read_body_json(resp2.into_body()).await;
    assert_eq!(body["data"]["id"], id);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_visits(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/milk/visits", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["data"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_quality(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/milk/quality", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["data"].is_array());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_milk_requires_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder()
        .uri("/api/v1/milk/day-productions")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
}
