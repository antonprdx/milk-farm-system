use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct GrazingData {
    pub id: i32,
    pub date: NaiveDate,
    pub animal_count: Option<i32>,
    pub pasture_time: Option<i32>,
    pub lactation_period: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct GrazingFilter {
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
