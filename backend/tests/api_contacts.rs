mod common;

use axum::body::Body;
use axum::http::StatusCode;
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_list_contacts_requires_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder()
        .uri("/api/v1/contacts")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_contacts_crud(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let token = admin_token();

    let create_req = auth_request_with_body(
        "POST",
        "/api/v1/contacts",
        &token,
        json!({
            "name": "Ivan",
            "active": true,
            "phone_cell": "+79991234567",
            "email": "ivan@test.com"
        }),
    );
    let resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Ivan");
    let id = body["data"]["id"].as_i64().unwrap();

    let list_req = auth_request("GET", "/api/v1/contacts", &token);
    let resp = app.clone().oneshot(list_req).await.unwrap();
    let list_body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(list_body["data"].as_array().unwrap().len(), 1);

    let update_req = auth_request_with_body(
        "PUT",
        &format!("/api/v1/contacts/{}", id),
        &token,
        json!({
            "name": "Ivan Updated"
        }),
    );
    let resp = app.clone().oneshot(update_req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Ivan Updated");

    let delete_req = auth_request("DELETE", &format!("/api/v1/contacts/{}", id), &token);
    let resp = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let list_req2 = auth_request("GET", "/api/v1/contacts", &token);
    let resp = app.oneshot(list_req2).await.unwrap();
    let list_body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(list_body["data"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_contact_not_found(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("DELETE", "/api/v1/contacts/99999", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
