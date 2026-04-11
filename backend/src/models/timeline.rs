use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TimelineEvent {
    pub date: NaiveDate,
    pub event_type: String,
    pub description: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TimelineFilter {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TimelineResponse {
    pub data: Vec<TimelineEvent>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
