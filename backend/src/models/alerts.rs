use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "alert_severity", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "alert_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "alert_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlertCategory {
    MilkDrop,
    HighScc,
    ActivityDrop,
    LowFeed,
    NoMilking,
    KetosisRisk,
    MastitisRisk,
    ExpectedCalving,
    EquipmentAnomaly,
    Other,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AlertRecord {
    pub id: i32,
    pub category: AlertCategory,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub animal_id: Option<i32>,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub detected_at: String,
    pub acknowledged_at: Option<String>,
    pub resolved_at: Option<String>,
    pub animal_name: Option<String>,
    pub life_number: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AlertFilter {
    pub status: Option<String>,
    pub category: Option<String>,
    pub severity: Option<String>,
    pub animal_id: Option<i32>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AlertsListResponse {
    pub data: Vec<AlertRecord>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ActiveAlertsSummary {
    pub total_active: i64,
    pub critical_count: i64,
    pub warning_count: i64,
    pub info_count: i64,
    pub by_category: serde_json::Value,
}
