mod common;

use axum::http::StatusCode;
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_list_locations_empty(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/locations", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_location(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/locations",
        &admin_token(),
        json!({ "name": "Barn A", "location_type": "barn" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Barn A");
    assert_eq!(body["data"]["location_type"], "barn");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_location_requires_admin(pool: sqlx::PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/locations",
        &user_token(),
        json!({ "name": "Barn A" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_location_validation(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/locations",
        &admin_token(),
        json!({ "name": "" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_location(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/locations",
        &admin_token(),
        json!({ "name": "Barn A", "location_type": "barn" }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let created: Value = read_body_json(resp.into_body()).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let update_req = auth_request_with_body(
        "PUT",
        &format!("/api/v1/locations/{}", id),
        &admin_token(),
        json!({ "name": "Barn B", "location_type": "pasture" }),
    );
    let resp = app.oneshot(update_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Barn B");
    assert_eq!(body["data"]["location_type"], "pasture");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_location_not_found(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "PUT",
        "/api/v1/locations/999",
        &admin_token(),
        json!({ "name": "X" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_location(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/locations",
        &admin_token(),
        json!({ "name": "ToDelete" }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let created: Value = read_body_json(resp.into_body()).await;
    let id = created["data"]["id"].as_i64().unwrap();

    let delete_req = auth_request("DELETE", &format!("/api/v1/locations/{}", id), &admin_token());
    let resp = app.oneshot(delete_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_location_not_found(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("DELETE", "/api/v1/locations/999", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_locations_require_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder()
        .uri("/api/v1/locations")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
