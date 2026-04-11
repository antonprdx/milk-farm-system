mod common;

use axum::http::StatusCode;
use milk_farm_backend::create_app;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::*;

fn make_app(pool: sqlx::PgPool) -> axum::Router {
    create_app(app_state(pool))
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_cannot_delete_self(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool.clone());

    let admin_id: (i32,) =
        sqlx::query_as("SELECT id FROM users WHERE username = 'admin'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let req = auth_request("DELETE", &format!("/api/v1/settings/users/{}", admin_id.0), &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["error"].as_str().unwrap().contains("own account"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_can_delete_other_user(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    seed_test_user(&pool).await;
    let app = make_app(pool.clone());

    let user_id: (i32,) =
        sqlx::query_as("SELECT id FROM users WHERE username = 'testuser'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let req = auth_request("DELETE", &format!("/api/v1/settings/users/{}", user_id.0), &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let deleted: Option<(i32,)> =
        sqlx::query_as("SELECT id FROM users WHERE id = $1")
            .bind(user_id.0)
            .fetch_optional(&pool)
            .await
            .unwrap();
    assert!(deleted.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_nonexistent_user(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);

    let req = auth_request("DELETE", "/api/v1/settings/users/99999", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_token_revoked_on_user_deletion(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    seed_test_user(&pool).await;
    let app = make_app(pool.clone());

    let user_id: (i32,) =
        sqlx::query_as("SELECT id FROM users WHERE username = 'testuser'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let token = user_token();
    let req = auth_request("GET", "/api/v1/animals", &token);
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let del_req = auth_request("DELETE", &format!("/api/v1/settings/users/{}", user_id.0), &admin_token());
    app.clone().oneshot(del_req).await.unwrap();

    let req = auth_request("GET", "/api/v1/animals", &token);
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_jwt_ttl_settings(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);

    let req = auth_request("GET", "/api/v1/settings/jwt-ttl", &admin_token());
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["jwt_access_ttl_secs"].is_number());
    assert!(body["jwt_refresh_ttl_secs"].is_number());

    let req = auth_request_with_body(
        "PUT",
        "/api/v1/settings/jwt-ttl",
        &admin_token(),
        json!({
            "jwt_access_ttl_secs": 1800,
            "jwt_refresh_ttl_secs": 1209600
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["jwt_access_ttl_secs"], 1800);
    assert_eq!(body["jwt_refresh_ttl_secs"], 1209600);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_jwt_ttl_requires_admin(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    seed_test_user(&pool).await;
    let app = make_app(pool);

    let req = auth_request("GET", "/api/v1/settings/jwt-ttl", &user_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_alert_thresholds_crud(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);

    let req = auth_request("GET", "/api/v1/settings/alert-thresholds", &admin_token());
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = auth_request_with_body(
        "PUT",
        "/api/v1/settings/alert-thresholds",
        &admin_token(),
        json!({
            "alert_min_milk": 10.0,
            "alert_max_scc": 500.0,
            "alert_days_before_calving": 14,
            "alert_activity_drop_pct": 30
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["alert_min_milk"], 10.0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_system_info(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);

    let req = auth_request("GET", "/api/v1/settings/system-info", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert!(body["db_size_mb"].is_number());
    assert!(body["total_users"].is_number());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_change_password_revokes_token(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool.clone());

    let req = auth_request_with_body(
        "POST",
        "/api/v1/auth/login",
        "",
        json!({ "username": "admin", "password": "admin12345" }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = auth_request_with_body(
        "POST",
        "/api/v1/settings/password",
        &admin_token(),
        json!({
            "old_password": "admin12345",
            "new_password": "newpassword123"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let revoked: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM revoked_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(revoked.0 > 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_role(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    seed_test_user(&pool).await;
    let app = make_app(pool.clone());

    let user_id: (i32,) =
        sqlx::query_as("SELECT id FROM users WHERE username = 'testuser'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let req = auth_request_with_body(
        "PUT",
        &format!("/api/v1/settings/users/{}/role", user_id.0),
        &admin_token(),
        json!({ "role": "admin" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let role: (String,) =
        sqlx::query_as("SELECT role FROM users WHERE id = $1")
            .bind(user_id.0)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(role.0, "admin");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_preferences_crud(pool: sqlx::PgPool) {
    seed_admin_user(&pool).await;
    let app = make_app(pool);

    let req = auth_request("GET", "/api/v1/settings/preferences", &admin_token());
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = auth_request_with_body(
        "PUT",
        "/api/v1/settings/preferences",
        &admin_token(),
        json!({
            "language": "en",
            "theme": "dark",
            "page_size": 50
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["page_size"], 50);

    let req = auth_request("GET", "/api/v1/settings/preferences", &admin_token());
    let resp = app.oneshot(req).await.unwrap();
    let body: Value = read_body_json(resp.into_body()).await;
    assert_eq!(body["page_size"], 50);
}
