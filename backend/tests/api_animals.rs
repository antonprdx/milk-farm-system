mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_list_animals_requires_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = Request::builder()
        .uri("/api/v1/animals")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_animals_empty(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/animals", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["total"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_animal(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request_with_body(
        "POST",
        "/api/v1/animals",
        &admin_token(),
        json!({
            "gender": "female",
            "birth_date": "2020-03-15",
            "name": "Burenka"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Burenka");
    assert_eq!(body["data"]["gender"], "female");
    assert!(body["data"]["active"].as_bool().unwrap());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_get_animal_by_id(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/animals",
        &admin_token(),
        json!({
            "gender": "female",
            "birth_date": "2020-01-01"
        }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    let id = body["data"]["id"].as_i64().unwrap();

    let get_req = auth_request("GET", &format!("/api/v1/animals/{}", id), &admin_token());
    let resp2 = app.oneshot(get_req).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let body2: Value = read_body_json(resp2.into_body()).await;
    assert_eq!(body2["data"]["id"], id);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_get_animal_not_found(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("GET", "/api/v1/animals/99999", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_animal(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/animals",
        &admin_token(),
        json!({
            "gender": "female",
            "birth_date": "2020-01-01",
            "name": "Old"
        }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let id = resp_json(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap();

    let update_req = auth_request_with_body(
        "PUT",
        &format!("/api/v1/animals/{}", id),
        &admin_token(),
        json!({
            "name": "New",
            "active": false
        }),
    );
    let resp2 = app.oneshot(update_req).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let body: Value = read_body_json(resp2.into_body()).await;
    assert_eq!(body["data"]["name"], "New");
    assert!(!body["data"]["active"].as_bool().unwrap());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_animal(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/animals",
        &admin_token(),
        json!({
            "gender": "male",
            "birth_date": "2021-06-01"
        }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    let id = resp_json(resp.into_body()).await["data"]["id"]
        .as_i64()
        .unwrap();

    let delete_req = auth_request("DELETE", &format!("/api/v1/animals/{}", id), &admin_token());
    let resp2 = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);

    let get_req = auth_request("GET", &format!("/api/v1/animals/{}", id), &admin_token());
    let resp3 = app.oneshot(get_req).await.unwrap();
    assert_eq!(resp3.status(), StatusCode::OK);
    let body3: Value = read_body_json(resp3.into_body()).await;
    assert!(!body3["data"]["active"].as_bool().unwrap());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_animals_with_filters(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    app.clone()
        .oneshot(auth_request_with_body(
            "POST",
            "/api/v1/animals",
            &admin_token(),
            json!({
                "gender": "female", "birth_date": "2020-01-01"
            }),
        ))
        .await
        .unwrap();
    app.clone()
        .oneshot(auth_request_with_body(
            "POST",
            "/api/v1/animals",
            &admin_token(),
            json!({
                "gender": "male", "birth_date": "2021-01-01"
            }),
        ))
        .await
        .unwrap();

    let req = auth_request("GET", "/api/v1/animals?gender=female", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["total"], 1);
}

async fn resp_json(body: Body) -> Value {
    read_body_json(body).await
}
