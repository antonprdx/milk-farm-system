pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod openapi;
pub mod seed;
pub mod services;
pub mod state;
pub mod validation;

use state::AppStateInner;
use utoipa::OpenApi;

pub async fn seed_admin(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = 'admin')")
            .fetch_one(pool)
            .await?;
    if !exists {
        let hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST)?;
        sqlx::query("INSERT INTO users (username, password_hash, role, must_change_password) VALUES ('admin', $1, 'admin', true)")
            .bind(&hash)
            .execute(pool)
            .await?;
        tracing::info!("Admin user created (change default password after first login)");
    }
    Ok(())
}

pub fn create_app(state: std::sync::Arc<AppStateInner>) -> axum::Router {
    use axum::http::Method;
    use axum::http::header::{AUTHORIZATION, CONTENT_TYPE, COOKIE};
    use tower_http::compression::CompressionLayer;
    use tower_http::cors::{AllowOrigin, CorsLayer};
    use tower_http::trace::TraceLayer;

    let origins: Vec<_> = state
        .config
        .cors_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, COOKIE])
        .allow_credentials(true);

    let api_routes = handlers::routes();

    let api_doc = crate::openapi::ApiDoc::openapi();
    let swagger =
        utoipa_swagger_ui::SwaggerUi::new("/api/v1/docs").url("/api/v1/docs/openapi.json", api_doc);

    let rate_limit =
        crate::middleware::rate_limit::RateLimitLayer::new(100, 60, state.config.trust_proxy);
    let request_id = crate::middleware::request_id::RequestIdLayer;
    let metrics_layer = crate::middleware::metrics::MetricsLayer;

    axum::Router::new()
        .merge(swagger)
        .nest("/api/v1", api_routes)
        .route("/metrics", axum::routing::get(metrics_endpoint))
        .layer(metrics_layer)
        .layer(request_id)
        .layer(rate_limit)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn metrics_endpoint() -> String {
    crate::middleware::metrics::gather()
}
