use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct FeedDayAmount {
    pub id: i32,
    pub animal_id: i32,
    pub feed_date: NaiveDate,
    pub feed_number: i32,
    pub total: f64,
    pub rest_feed: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct FeedVisit {
    pub id: i32,
    pub animal_id: i32,
    pub visit_datetime: chrono::DateTime<chrono::Utc>,
    pub feed_number: Option<i32>,
    pub amount: Option<f64>,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct FeedType {
    pub id: i32,
    pub number_of_feed_type: i32,
    pub feed_type: String,
    pub name: String,
    pub description: Option<String>,
    pub dry_matter_percentage: f64,
    pub stock_attention_level: Option<i32>,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct FeedGroup {
    pub id: i32,
    pub name: String,
    pub min_milk_yield: Option<f64>,
    pub max_milk_yield: Option<f64>,
    pub avg_milk_yield: Option<f64>,
    pub avg_milk_fat: Option<f64>,
    pub avg_milk_protein: Option<f64>,
    pub avg_weight: Option<f64>,
    pub max_robot_feed_types: Option<i32>,
    pub max_feed_intake_robot: Option<f64>,
    pub min_feed_intake_robot: Option<f64>,
    pub number_of_cows: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateFeedDayAmount {
    pub animal_id: i32,
    pub feed_date: NaiveDate,
    pub feed_number: i32,
    pub total: f64,
    pub rest_feed: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct FeedFilter {
    pub animal_id: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
