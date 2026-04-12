use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::system_settings::*;

static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

pub fn start_time() -> &'static std::time::Instant {
    START_TIME.get_or_init(std::time::Instant::now)
}

async fn get_value(pool: &PgPool, key: &str) -> Result<String, AppError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM system_settings WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

    row.map(|r| r.0)
        .ok_or_else(|| AppError::NotFound(format!("Настройка {} не найдена", key)))
}

async fn set_value(pool: &PgPool, key: &str, value: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO system_settings (key, value, updated_at) VALUES ($1, $2, NOW())
         ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(())
}

pub async fn get_jwt_ttl(pool: &PgPool) -> Result<JwtTtlSettings, AppError> {
    Ok(JwtTtlSettings {
        jwt_access_ttl_secs: get_value(pool, "jwt_access_ttl_secs")
            .await?
            .parse()
            .unwrap_or(900),
        jwt_refresh_ttl_secs: get_value(pool, "jwt_refresh_ttl_secs")
            .await?
            .parse()
            .unwrap_or(604800),
    })
}

pub async fn update_jwt_ttl(pool: &PgPool, req: &UpdateJwtTtl) -> Result<JwtTtlSettings, AppError> {
    if let Some(v) = req.jwt_access_ttl_secs {
        if !(60..=86400).contains(&v) {
            return Err(AppError::BadRequest(
                "jwt_access_ttl_secs must be 60-86400".into(),
            ));
        }
        set_value(pool, "jwt_access_ttl_secs", &v.to_string()).await?;
    }
    if let Some(v) = req.jwt_refresh_ttl_secs {
        if !(3600..=2592000).contains(&v) {
            return Err(AppError::BadRequest(
                "jwt_refresh_ttl_secs must be 3600-2592000".into(),
            ));
        }
        set_value(pool, "jwt_refresh_ttl_secs", &v.to_string()).await?;
    }
    get_jwt_ttl(pool).await
}

pub async fn get_alert_thresholds(pool: &PgPool) -> Result<AlertThresholds, AppError> {
    Ok(AlertThresholds {
        alert_min_milk: get_value(pool, "alert_min_milk")
            .await?
            .parse()
            .unwrap_or(5.0),
        alert_max_scc: get_value(pool, "alert_max_scc")
            .await?
            .parse()
            .unwrap_or(400.0),
        alert_days_before_calving: get_value(pool, "alert_days_before_calving")
            .await?
            .parse()
            .unwrap_or(14),
        alert_activity_drop_pct: get_value(pool, "alert_activity_drop_pct")
            .await?
            .parse()
            .unwrap_or(30),
    })
}

pub async fn update_alert_thresholds(
    pool: &PgPool,
    req: &UpdateAlertThresholds,
) -> Result<AlertThresholds, AppError> {
    if let Some(v) = req.alert_min_milk {
        set_value(pool, "alert_min_milk", &v.to_string()).await?;
    }
    if let Some(v) = req.alert_max_scc {
        set_value(pool, "alert_max_scc", &v.to_string()).await?;
    }
    if let Some(v) = req.alert_days_before_calving {
        set_value(pool, "alert_days_before_calving", &v.to_string()).await?;
    }
    if let Some(v) = req.alert_activity_drop_pct {
        set_value(pool, "alert_activity_drop_pct", &v.to_string()).await?;
    }
    get_alert_thresholds(pool).await
}

pub async fn get_system_info(pool: &PgPool) -> Result<SystemInfo, AppError> {
    let (db_size, animals, milk, repro, users) = tokio::join!(
        sqlx::query_as::<_, (f64,)>("SELECT (pg_database_size(current_database())::double precision / 1024.0 / 1024.0)::double precision")
            .fetch_one(pool),
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM animals")
            .fetch_one(pool),
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM milk_day_productions")
            .fetch_one(pool),
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(SUM(c)::bigint, 0) FROM (SELECT COUNT(*) as c FROM calvings UNION ALL SELECT COUNT(*) FROM inseminations UNION ALL SELECT COUNT(*) FROM pregnancies UNION ALL SELECT COUNT(*) FROM heats UNION ALL SELECT COUNT(*) FROM dry_offs) sub")
            .fetch_one(pool),
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM users")
            .fetch_one(pool),
    );
    let db_size = db_size.map_err(AppError::Database)?;
    let animals = animals.map_err(AppError::Database)?;
    let milk = milk.map_err(AppError::Database)?;
    let repro = repro.map_err(AppError::Database)?;
    let users = users.map_err(AppError::Database)?;

    Ok(SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: start_time().elapsed().as_secs(),
        db_size_mb: (db_size.0 * 100.0).round() / 100.0,
        total_animals: animals.0,
        total_milk_records: milk.0,
        total_reproduction_records: repro.0,
        total_users: users.0,
    })
}

pub async fn generate_backup(_pool: &PgPool) -> Result<Vec<u8>, AppError> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| AppError::Internal(anyhow::anyhow!("DATABASE_URL not set")))?;

    let parsed = url::Url::parse(&db_url)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid DATABASE_URL: {e}")))?;

    let host = parsed.host_str().unwrap_or("localhost");
    let port = parsed.port().unwrap_or(5432);
    let user = parsed.username();
    let db_name = parsed.path().trim_start_matches('/');

    let password = parsed.password().unwrap_or("");

    let output = std::process::Command::new("pg_dump")
        .args(["--format=plain", "--no-owner", "--no-acl"])
        .arg("--host")
        .arg(host)
        .arg("--port")
        .arg(port.to_string())
        .arg("--username")
        .arg(user)
        .arg("--dbname")
        .arg(db_name)
        .env("PGPASSWORD", password)
        .output()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("pg_dump failed: {e}")))?;

    if !output.status.success() {
        tracing::error!("pg_dump failed with status {}", output.status);
        return Err(AppError::Internal(anyhow::anyhow!(
            "pg_dump failed with exit code {}",
            output.status.code().unwrap_or(-1)
        )));
    }

    let size = output.stdout.len();
    if size > 100 * 1024 * 1024 {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Backup too large: {} bytes",
            size
        )));
    }

    Ok(output.stdout)
}
