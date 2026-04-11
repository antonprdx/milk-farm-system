use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSystemSetting {
    pub value: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemInfo {
    pub version: String,
    pub uptime_secs: u64,
    pub db_size_mb: f64,
    pub total_animals: i64,
    pub total_milk_records: i64,
    pub total_reproduction_records: i64,
    pub total_users: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AlertThresholds {
    pub alert_min_milk: f64,
    pub alert_max_scc: f64,
    pub alert_days_before_calving: i32,
    pub alert_activity_drop_pct: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAlertThresholds {
    pub alert_min_milk: Option<f64>,
    pub alert_max_scc: Option<f64>,
    pub alert_days_before_calving: Option<i32>,
    pub alert_activity_drop_pct: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateJwtTtl {
    pub jwt_access_ttl_secs: Option<u64>,
    pub jwt_refresh_ttl_secs: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JwtTtlSettings {
    pub jwt_access_ttl_secs: u64,
    pub jwt_refresh_ttl_secs: u64,
}
