mod common;

use axum::body::Body;
use axum::http::StatusCode;
use serde_json::{json, Value};
use tower::ServiceExt;
use milk_farm_backend::create_app;

use common::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_list_contacts_requires_auth(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = axum::http::Request::builder().uri("/api/contacts").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_contacts_crud(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let token = admin_token();

    let create_req = auth_request_with_body("POST", "/api/contacts", &token, json!({
        "name": "Ivan",
        "active": true,
        "phone_cell": "+79991234567",
        "email": "ivan@test.com"
    }));
    let resp = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Ivan");
    let id = body["data"]["id"].as_i64().unwrap();

    let list_req = auth_request("GET", "/api/contacts", &token);
    let resp = app.clone().oneshot(list_req).await.unwrap();
    let list_body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(list_body["data"].as_array().unwrap().len(), 1);

    let update_req = auth_request_with_body("PUT", &format!("/api/contacts/{}", id), &token, json!({
        "name": "Ivan Updated"
    }));
    let resp = app.clone().oneshot(update_req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["data"]["name"], "Ivan Updated");

    let delete_req = auth_request("DELETE", &format!("/api/contacts/{}", id), &token);
    let resp = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let list_req2 = auth_request("GET", "/api/contacts", &token);
    let resp = app.oneshot(list_req2).await.unwrap();
    let list_body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(list_body["data"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_contact_not_found(pool: sqlx::PgPool) {
    let app = create_app(app_state(pool));
    let req = auth_request("DELETE", "/api/contacts/99999", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
