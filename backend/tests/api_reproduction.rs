mod common;

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
async fn test_list_calvings_empty(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/reproduction/calvings", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_calving(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;

    let req = auth_request_with_body(
        "POST",
        "/api/reproduction/calvings",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "calving_date": "2024-06-01",
            "remarks": "Normal"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["animal_id"], animal_id);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_get_calving_by_id(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;

    let create_req = auth_request_with_body(
        "POST",
        "/api/reproduction/calvings",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "calving_date": "2024-06-01"
        }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let id = read_body_json::<Value>(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap();

    let get_req = auth_request(
        "GET",
        &format!("/api/reproduction/calvings/{}", id),
        &admin_token(),
    );
    let resp2 = app.oneshot(get_req).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_insemination(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;

    let req = auth_request_with_body(
        "POST",
        "/api/reproduction/inseminations",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "insemination_date": "2024-03-15",
            "sire_code": "SIRE001"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["sire_code"], "SIRE001");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_pregnancy(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;

    let req = auth_request_with_body(
        "POST",
        "/api/reproduction/pregnancies",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "pregnancy_date": "2024-04-15",
            "pregnancy_type": "ultrasound"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["pregnancy_type"], "ultrasound");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_heat(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;

    let req = auth_request_with_body(
        "POST",
        "/api/reproduction/heats",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "heat_date": "2024-05-01"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["animal_id"], animal_id);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_dryoff(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let animal_id = create_test_animal(&app).await;

    let req = auth_request_with_body(
        "POST",
        "/api/reproduction/dryoffs",
        &admin_token(),
        json!({
            "animal_id": animal_id,
            "dry_off_date": "2024-07-01"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["animal_id"], animal_id);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_current_status(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/reproduction/status", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["data"].is_array());
}
