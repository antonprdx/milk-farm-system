use std::sync::Arc;

use milk_farm_backend::{
    config, create_app, seed_admin, services::system_settings_service, state::AppStateInner,
};

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_default();
    if log_format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    let cfg = config::Config::from_env()?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(
            std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(20),
        )
        .min_connections(
            std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
        )
        .acquire_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(&cfg.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    seed_admin(&pool).await?;

    system_settings_service::start_time();
    milk_farm_backend::middleware::metrics::init();

    let lely_runtime = milk_farm_backend::state::LelyRuntime::new(
        milk_farm_backend::lely::service::load_config(&pool, &cfg.lely_encryption_key)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Не удалось загрузить Lely config из БД, используем env vars");
                cfg.lely_env.clone()
            }),
    );

    let state = Arc::new(AppStateInner {
        pool,
        config: cfg.clone(),
        lely: Arc::new(lely_runtime),
    });

    {
        let lc = state.lely.get_config();
        if lc.enabled {
            milk_farm_backend::lely::sync::start_sync_scheduler(state.clone());
        } else {
            tracing::info!("Интеграция Lely отключена");
        }
    }

    let app = create_app(state);

    let addr = cfg.addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server running on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            shutdown_signal().await;
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            tracing::warn!("Graceful shutdown timed out after 30s");
        })
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutting down gracefully...");
}
