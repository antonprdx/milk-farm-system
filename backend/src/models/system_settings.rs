use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::errors::AppError;

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

impl UpdateAlertThresholds {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        if let Some(v) = self.alert_min_milk
            && !(0.0..=100.0).contains(&v)
        {
            return Err(AppError::BadRequest(
                "Минимальный удой должен быть от 0 до 100".into(),
            ));
        }
        if let Some(v) = self.alert_max_scc
            && !(0.0..=10000.0).contains(&v)
        {
            return Err(AppError::BadRequest(
                "Макс. соматических клеток должен быть от 0 до 10000".into(),
            ));
        }
        opt_positive_i32(&self.alert_days_before_calving, "Дни до отёла")?;
        if let Some(v) = self.alert_days_before_calving
            && v > 90
        {
            return Err(AppError::BadRequest(
                "Дни до отёла не могут быть больше 90".into(),
            ));
        }
        if let Some(v) = self.alert_activity_drop_pct
            && !(0..=100).contains(&v)
        {
            return Err(AppError::BadRequest(
                "Снижение активности должно быть от 0 до 100".into(),
            ));
        }
        Ok(())
    }
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
