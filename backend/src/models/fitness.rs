use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: i32,
    pub animal_id: i32,
    pub activity_datetime: chrono::DateTime<chrono::Utc>,
    pub activity_counter: Option<i32>,
    pub heat_attention: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Rumination {
    pub id: i32,
    pub animal_id: i32,
    pub date: NaiveDate,
    pub eating_seconds: Option<i32>,
    pub rumination_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FitnessFilter {
    pub animal_id: Option<i32>,
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
