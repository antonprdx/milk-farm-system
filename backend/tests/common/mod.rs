use std::sync::Arc;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use milk_farm_backend::config::Config;
use milk_farm_backend::middleware::auth::create_access_token;
use milk_farm_backend::state::AppStateInner;
use serde::de::DeserializeOwned;
use sqlx::PgPool;

pub fn test_config() -> Config {
    Config {
        database_url: String::new(),
        jwt_secret: "test_secret_key_32_characters_long!".to_string(),
        host: "127.0.0.1".to_string(),
        port: 3000,
        cors_origins: vec!["http://localhost:5173".to_string()],
        secure_cookies: false,
        jwt_access_ttl_secs: 900,
        jwt_refresh_ttl_secs: 604800,
        trust_proxy: false,
    }
}

pub fn app_state(pool: PgPool) -> Arc<AppStateInner> {
    Arc::new(AppStateInner {
        pool,
        config: test_config(),
    })
}

pub fn admin_token() -> String {
    create_access_token("admin", "admin", false, &test_config().jwt_secret, 900).unwrap()
}

#[allow(dead_code)]
pub fn user_token() -> String {
    create_access_token("testuser", "user", false, &test_config().jwt_secret, 900).unwrap()
}

pub fn auth_request(method: &str, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::empty())
        .unwrap()
}

#[allow(dead_code)]
pub fn auth_request_with_body(
    method: &str,
    uri: &str,
    token: &str,
    body: impl serde::Serialize,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

pub async fn read_body_json<T: DeserializeOwned>(body: Body) -> T {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[allow(dead_code)]
pub async fn read_body_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[allow(dead_code)]
pub async fn seed_admin_user(pool: &PgPool) {
    let hash = bcrypt::hash("admin12345", bcrypt::DEFAULT_COST).unwrap();
    sqlx::query("INSERT INTO users (username, password_hash, role) VALUES ('admin', $1, 'admin') ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash")
        .bind(&hash)
        .execute(pool)
        .await
        .unwrap();
}

#[allow(dead_code)]
pub async fn seed_test_user(pool: &PgPool) {
    let hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
    sqlx::query("INSERT INTO users (username, password_hash, role) VALUES ('testuser', $1, 'user') ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash")
        .bind(&hash)
        .execute(pool)
        .await
        .unwrap();
}

#[allow(dead_code)]
pub async fn seed_animal(pool: &PgPool) -> i32 {
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO animals (gender, birth_date, active) VALUES ('female', '2020-01-01'::date, true) RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}

#[allow(dead_code)]
pub async fn seed_male_animal(pool: &PgPool) -> i32 {
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO animals (gender, birth_date, active) VALUES ('male', '2021-06-15'::date, true) RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}

#[allow(dead_code)]
pub async fn seed_location(pool: &PgPool) -> i32 {
    let row: (i32,) =
        sqlx::query_as("INSERT INTO locations (name) VALUES ('Test Barn') RETURNING id")
            .fetch_one(pool)
            .await
            .unwrap();
    row.0
}

#[allow(dead_code)]
pub async fn seed_feed_type(pool: &PgPool) -> i32 {
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO feed_types (number_of_feed_type, feed_type, name, dry_matter_percentage, price) VALUES (1, 'grain', 'Test Feed', 85.0, 10.0) RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}
