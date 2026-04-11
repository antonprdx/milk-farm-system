mod common;

use axum::body::Body;
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_list_bulk_tank_empty(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/bulk-tank", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_bulk_tank(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/bulk-tank",
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
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["fat"], 3.8);
    assert_eq!(body["data"]["protein"], 3.2);
    assert_eq!(body["data"]["lactose"], 4.6);
    assert_eq!(body["data"]["scc"], 150);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_get_bulk_tank_by_id(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/bulk-tank",
        &admin_token(),
        json!({
            "date": "2025-01-15",
            "fat": 3.8,
            "protein": 3.2
        }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let id = read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap();

    let get_req = auth_request("GET", &format!("/api/v1/bulk-tank/{}", id), &admin_token());
    let resp2 = app.oneshot(get_req).await.unwrap();
    let body: Value = read_body_json(resp2.into_body()).await;
    assert_eq!(body["data"]["id"], id);
    assert_eq!(body["data"]["fat"], 3.8);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_bulk_tank(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/bulk-tank",
        &admin_token(),
        json!({
            "date": "2025-01-15",
            "fat": 3.8,
            "protein": 3.2
        }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let id = read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap();

    let update_req = auth_request_with_body(
        "PUT",
        &format!("/api/v1/bulk-tank/{}", id),
        &admin_token(),
        json!({
            "fat": 4.1,
            "protein": 3.5
        }),
    );
    let resp2 = app.oneshot(update_req).await.unwrap();
    let body: Value = read_body_json(resp2.into_body()).await;
    assert_eq!(body["data"]["fat"], 4.1);
    assert_eq!(body["data"]["protein"], 3.5);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_bulk_tank(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/bulk-tank",
        &admin_token(),
        json!({
            "date": "2025-01-15",
            "fat": 3.8,
            "protein": 3.2
        }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let id = read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap();

    let del_req = auth_request("DELETE", &format!("/api/v1/bulk-tank/{}", id), &admin_token());
    let resp2 = app.clone().oneshot(del_req).await.unwrap();
    assert_eq!(resp2.status(), axum::http::StatusCode::OK);

    let get_req = auth_request("GET", &format!("/api/v1/bulk-tank/{}", id), &admin_token());
    let resp3 = app.oneshot(get_req).await.unwrap();
    assert_eq!(resp3.status(), axum::http::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_bulk_tank_requires_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder()
        .uri("/api/v1/bulk-tank")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);
}
