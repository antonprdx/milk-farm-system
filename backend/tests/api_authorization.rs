mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

use common::*;
use milk_farm_backend::create_app;

async fn seed_milk_production(app: &axum::Router, animal_id: i64) -> i64 {
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
    let resp = app.clone().oneshot(req).await.unwrap();
    read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap()
}

async fn seed_calving(app: &axum::Router, animal_id: i64) -> i64 {
    let req = auth_request_with_body(
        "POST",
        "/api/v1/reproduction/calvings",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "calving_date": "2024-06-01"
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap()
}

async fn seed_insemination(app: &axum::Router, animal_id: i64) -> i64 {
    let req = auth_request_with_body(
        "POST",
        "/api/v1/reproduction/inseminations",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "insemination_date": "2024-03-15",
            "sire_code": "SIRE001"
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap()
}

async fn seed_contact(app: &axum::Router) -> i64 {
    let req = auth_request_with_body(
        "POST",
        "/api/v1/contacts",
        &admin_token(),
        json!({
            "name": "Test Contact",
            "active": true,
            "phone_cell": "+79990000000",
            "email": "test@example.com"
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap()
}

async fn seed_bulk_tank(app: &axum::Router) -> i64 {
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
    let resp = app.clone().oneshot(req).await.unwrap();
    read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_create_animals(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let token = user_token();

    let req = auth_request_with_body(
        "POST",
        "/api/v1/animals",
        &token,
        json!({
            "gender": "female",
            "birth_date": "2020-01-01"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_update_animals(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;
    let token = user_token();

    let req = auth_request_with_body(
        "PUT",
        &format!("/api/v1/animals/{}", animal_id),
        &token,
        json!({
            "name": "Hacked"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_delete_animals(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;
    let token = user_token();

    let req = auth_request("DELETE", &format!("/api/v1/animals/{}", animal_id), &token);
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_delete_milk_productions(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;
    let production_id = seed_milk_production(&app, animal_id).await;
    let token = user_token();

    let req = auth_request(
        "DELETE",
        &format!("/api/v1/milk/day-productions/{}", production_id),
        &token,
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_delete_calvings(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;
    let calving_id = seed_calving(&app, animal_id).await;
    let token = user_token();

    let req = auth_request(
        "DELETE",
        &format!("/api/v1/reproduction/calvings/{}", calving_id),
        &token,
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_delete_inseminations(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;
    let insemination_id = seed_insemination(&app, animal_id).await;
    let token = user_token();

    let req = auth_request(
        "DELETE",
        &format!("/api/v1/reproduction/inseminations/{}", insemination_id),
        &token,
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_delete_contacts(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let contact_id = seed_contact(&app).await;
    let token = user_token();

    let req = auth_request(
        "DELETE",
        &format!("/api/v1/contacts/{}", contact_id),
        &token,
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_delete_bulk_tank_tests(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let bt_id = seed_bulk_tank(&app).await;
    let token = user_token();

    let req = auth_request("DELETE", &format!("/api/v1/bulk-tank/{}", bt_id), &token);
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_cannot_register_users(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let token = user_token();

    let req = auth_request_with_body(
        "POST",
        "/api/v1/auth/register",
        &token,
        json!({
            "username": "newuser",
            "password": "password123"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_can_read_animals(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let token = user_token();

    let req = auth_request("GET", "/api/v1/animals", &token);
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_can_read_milk_productions(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let token = user_token();

    let req = auth_request("GET", "/api/v1/milk/day-productions", &token);
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_can_read_calvings(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let token = user_token();

    let req = auth_request("GET", "/api/v1/reproduction/calvings", &token);
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_can_read_inseminations(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let token = user_token();

    let req = auth_request("GET", "/api/v1/reproduction/inseminations", &token);
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_can_read_contacts(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let token = user_token();

    let req = auth_request("GET", "/api/v1/contacts", &token);
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_can_read_bulk_tank(pool: PgPool) {
    seed_test_user(&pool).await;
    let app = create_app(app_state(pool));
    let token = user_token();

    let req = auth_request("GET", "/api/v1/bulk-tank", &token);
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
