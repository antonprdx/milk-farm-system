mod common;

use axum::body::Body;
use axum::http::StatusCode;
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_create_feed_type(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/feed/types",
        &admin_token(),
        json!({
            "number_of_feed_type": 1,
            "feed_type": "grain",
            "name": "Test Grain",
            "dry_matter_percentage": 85.0,
            "price": 12.5
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Test Grain");
    assert_eq!(body["data"]["feed_type"], "grain");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_feed_type(pool: sqlx::PgPool) {
    let ft_id = seed_feed_type(&pool).await;
    let app = create_app(app_state(pool));

    let req = auth_request_with_body(
        "PUT",
        &format!("/api/v1/feed/types/{}", ft_id),
        &admin_token(),
        json!({ "name": "Updated Feed", "price": 15.0 }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Updated Feed");
    assert_eq!(body["data"]["price"], 15.0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_feed_type(pool: sqlx::PgPool) {
    let ft_id = seed_feed_type(&pool).await;
    let app = create_app(app_state(pool));

    let req = auth_request(
        "DELETE",
        &format!("/api/v1/feed/types/{}", ft_id),
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_nonexistent_feed_type(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("DELETE", "/api/v1/feed/types/99999", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_feed_group(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/feed/groups",
        &admin_token(),
        json!({
            "name": "High Yield",
            "min_milk_yield": 25.0,
            "max_milk_yield": 50.0
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "High Yield");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_feed_group(pool: sqlx::PgPool) {
    let row: (i32,) =
        sqlx::query_as("INSERT INTO feed_groups (name) VALUES ('Test Group') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let g_id = row.0;
    let app = create_app(app_state(pool));

    let req = auth_request_with_body(
        "PUT",
        &format!("/api/v1/feed/groups/{}", g_id),
        &admin_token(),
        json!({ "name": "Updated Group" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Updated Group");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_feed_group(pool: sqlx::PgPool) {
    let row: (i32,) =
        sqlx::query_as("INSERT INTO feed_groups (name) VALUES ('Delete Me') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let app = create_app(app_state(pool));

    let req = auth_request(
        "DELETE",
        &format!("/api/v1/feed/groups/{}", row.0),
        &admin_token(),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_feed_day_amount(pool: sqlx::PgPool) {
    let animal_id = seed_animal(&pool).await;
    let app = create_app(app_state(pool));

    let req = auth_request_with_body(
        "POST",
        "/api/v1/feed/day-amounts",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "feed_date": "2025-01-15",
            "feed_number": 1,
            "total": 12.5
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["total"], 12.5);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_feed_crud_requires_admin(pool: sqlx::PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/feed/types")
        .header("Authorization", format!("Bearer {}", user_token()))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "number_of_feed_type": 1,
                "feed_type": "grain",
                "name": "X",
                "dry_matter_percentage": 80.0,
                "price": 5.0
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_feed_type_validation_empty_name(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/feed/types",
        &admin_token(),
        json!({
            "number_of_feed_type": 1,
            "feed_type": "grain",
            "name": "",
            "dry_matter_percentage": 85.0,
            "price": 12.5
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_feed_type_validation_negative_price(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/feed/types",
        &admin_token(),
        json!({
            "number_of_feed_type": 1,
            "feed_type": "grain",
            "name": "Test",
            "dry_matter_percentage": 85.0,
            "price": -5.0
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_feed_group_validation_empty_name(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/feed/groups",
        &admin_token(),
        json!({
            "name": "",
            "min_milk_yield": 25.0
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_feed_day_amount_validation_negative_total(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/feed/day-amounts",
        &admin_token(),
        json!({
            "animal_id": 1,
            "feed_date": "2025-01-15",
            "feed_number": 1,
            "total": -10.0
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
