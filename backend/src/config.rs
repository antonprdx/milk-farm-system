use anyhow::Context;

#[derive(Debug, Clone)]
pub struct LelyConfig {
    pub enabled: bool,
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub farm_key: String,
    pub sync_interval_secs: u64,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub secure_cookies: bool,
    pub jwt_access_ttl_secs: u64,
    pub jwt_refresh_ttl_secs: u64,
    pub trust_proxy: bool,
    pub lely_encryption_key: String,
    pub lely_env: LelyConfig,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL not set")?;
        let jwt_secret = std::env::var("JWT_SECRET").context("JWT_SECRET not set")?;

        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters");
        }

        let cors_origins = std::env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            database_url,
            jwt_secret,
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            cors_origins,
            secure_cookies: std::env::var("SECURE_COOKIES")
                .unwrap_or_else(|_| "true".into())
                .parse()
                .unwrap_or(true),
            jwt_access_ttl_secs: std::env::var("JWT_ACCESS_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(900),
            jwt_refresh_ttl_secs: std::env::var("JWT_REFRESH_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(604800),
            trust_proxy: std::env::var("TRUST_PROXY")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
            lely_encryption_key: {
                let key = std::env::var("LELY_ENCRYPTION_KEY").context(
                    "LELY_ENCRYPTION_KEY not set — required for encrypting Lely credentials",
                )?;
                if key.len() < 32 {
                    anyhow::bail!(
                        "LELY_ENCRYPTION_KEY must be at least 32 characters, got {}",
                        key.len()
                    );
                }
                key
            },
            lely_env: LelyConfig {
                enabled: std::env::var("LELY_ENABLED")
                    .unwrap_or_else(|_| "false".into())
                    .parse()
                    .unwrap_or(false),
                base_url: std::env::var("LELY_BASE_URL").unwrap_or_else(|_| String::new()),
                username: std::env::var("LELY_USERNAME").unwrap_or_else(|_| String::new()),
                password: std::env::var("LELY_PASSWORD").unwrap_or_else(|_| String::new()),
                farm_key: std::env::var("LELY_FARM_KEY").unwrap_or_else(|_| String::new()),
                sync_interval_secs: std::env::var("LELY_SYNC_INTERVAL_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(300),
            },
        })
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
