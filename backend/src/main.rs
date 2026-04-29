use std::sync::Arc;

use milk_farm_backend::{
    config, create_app, handlers::events::create_event_bus, seed_admin, services::{ml_client::MlClient, system_settings_service}, state::AppStateInner,
};

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_default();
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_default();

    if !otlp_endpoint.is_empty() {
        init_tracing_with_otel(&log_format, &otlp_endpoint)?;
    } else if log_format == "json" {
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

    let redis = match redis::Client::open(cfg.redis_url.as_str()) {
        Ok(client) => {
            match tokio::time::timeout(
                std::time::Duration::from_secs(2),
                client.get_connection_manager(),
            )
            .await
            {
                Ok(Ok(mgr)) => {
                    tracing::info!("Connected to Redis at {}", cfg.redis_url);
                    Some(mgr)
                }
                Ok(Err(e)) => {
                    tracing::warn!("Redis connection failed: {}. Running without cache.", e);
                    None
                }
                Err(_) => {
                    tracing::warn!("Redis connection timed out. Running without cache.");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!("Redis config invalid: {}. Running without cache.", e);
            None
        }
    };

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
        ml: MlClient::from_env(),
        redis,
        event_bus: create_event_bus(),
    });

    {
        let lc = state.lely.get_config().await;
        if lc.enabled {
            milk_farm_backend::lely::sync::start_sync_scheduler(state.clone());
        } else {
            tracing::info!("Интеграция Lely отключена");
        }
    }

    {
        let cleanup_pool = state.pool.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                if let Err(e) =
                    milk_farm_backend::services::token_revocation_service::cleanup_expired(
                        &cleanup_pool,
                    )
                    .await
                {
                    tracing::warn!(error = %e, "Token cleanup failed");
                }
            }
        });
    }

    let app = create_app(state);

    let addr = cfg.addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server running on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
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

fn init_tracing_with_otel(log_format: &str, endpoint: &str) -> anyhow::Result<()> {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.to_string())
        .build()?;

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_simple_exporter(span_exporter)
        .build();

    let tracer = tracer_provider.tracer("milk-farm-backend");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let env_filter = EnvFilter::from_default_env();

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(otel_layer)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(otel_layer)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    Ok(())
}
