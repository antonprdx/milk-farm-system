use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct MilkDataPoint {
    pub date: NaiveDate,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct SccDataPoint {
    pub date: NaiveDate,
    pub scc: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LatestMetrics {
    pub avg_milk_30d: Option<f64>,
    pub last_scc: Option<i32>,
    pub avg_weight_30d: Option<f64>,
    pub avg_isk_30d: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReproductionSummary {
    pub last_calving_date: Option<NaiveDate>,
    pub total_inseminations: i64,
    pub expected_calving_date: Option<NaiveDate>,
    pub is_pregnant: bool,
    pub lactation_number: Option<i32>,
    pub days_in_milk: Option<i64>,
    pub is_dry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AnimalStats {
    pub milk_production_30d: Vec<MilkDataPoint>,
    pub scc_trend_90d: Vec<SccDataPoint>,
    pub latest_metrics: LatestMetrics,
    pub reproduction: ReproductionSummary,
}
