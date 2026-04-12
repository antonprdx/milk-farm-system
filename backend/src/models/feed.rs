use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::errors::AppError;

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

impl CreateFeedDayAmount {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        positive_i32(self.animal_id, "ID животного")?;
        positive_i32(self.feed_number, "Номер корма")?;
        non_negative_f64(self.total, "Количество корма")?;
        opt_non_negative_i32(&self.rest_feed, "Остаток корма")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateFeedType {
    pub number_of_feed_type: i32,
    pub feed_type: String,
    pub name: String,
    pub description: Option<String>,
    pub dry_matter_percentage: f64,
    pub stock_attention_level: Option<i32>,
    pub price: f64,
}

impl CreateFeedType {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        positive_i32(self.number_of_feed_type, "Номер типа корма")?;
        required_non_empty(&self.feed_type, "Тип корма")?;
        max_len(&self.feed_type, 50, "Тип корма")?;
        required_non_empty(&self.name, "Название")?;
        max_len(&self.name, 200, "Название")?;
        opt_max_len(&self.description, 500, "Описание")?;
        percentage_f64(self.dry_matter_percentage, "Процент сухого вещества")?;
        opt_positive_i32(&self.stock_attention_level, "Уровень остатка")?;
        non_negative_f64(self.price, "Цена")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateFeedType {
    pub number_of_feed_type: Option<i32>,
    pub feed_type: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub dry_matter_percentage: Option<f64>,
    pub stock_attention_level: Option<i32>,
    pub price: Option<f64>,
}

impl UpdateFeedType {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        opt_positive_i32(&self.number_of_feed_type, "Номер типа корма")?;
        if let Some(ref v) = self.feed_type {
            required_non_empty(v, "Тип корма")?;
            max_len(v, 50, "Тип корма")?;
        }
        if let Some(ref v) = self.name {
            required_non_empty(v, "Название")?;
            max_len(v, 200, "Название")?;
        }
        opt_max_len(&self.description, 500, "Описание")?;
        opt_percentage_f64(&self.dry_matter_percentage, "Процент сухого вещества")?;
        opt_positive_i32(&self.stock_attention_level, "Уровень остатка")?;
        opt_non_negative_f64(&self.price, "Цена")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateFeedGroup {
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

impl CreateFeedGroup {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        required_non_empty(&self.name, "Название группы")?;
        max_len(&self.name, 200, "Название группы")?;
        opt_non_negative_f64(&self.min_milk_yield, "Мин. удой")?;
        opt_non_negative_f64(&self.max_milk_yield, "Макс. удой")?;
        opt_non_negative_f64(&self.avg_milk_yield, "Средний удой")?;
        opt_percentage_f64(&self.avg_milk_fat, "Средний жир")?;
        opt_percentage_f64(&self.avg_milk_protein, "Средний белок")?;
        opt_non_negative_f64(&self.avg_weight, "Средний вес")?;
        opt_positive_i32(&self.max_robot_feed_types, "Макс. типов корма робота")?;
        opt_non_negative_f64(&self.max_feed_intake_robot, "Макс. потребление робота")?;
        opt_non_negative_f64(&self.min_feed_intake_robot, "Мин. потребление робота")?;
        opt_positive_i32(&self.number_of_cows, "Количество коров")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateFeedGroup {
    pub name: Option<String>,
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

impl UpdateFeedGroup {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        if let Some(ref v) = self.name {
            required_non_empty(v, "Название группы")?;
            max_len(v, 200, "Название группы")?;
        }
        opt_non_negative_f64(&self.min_milk_yield, "Мин. удой")?;
        opt_non_negative_f64(&self.max_milk_yield, "Макс. удой")?;
        opt_non_negative_f64(&self.avg_milk_yield, "Средний удой")?;
        opt_percentage_f64(&self.avg_milk_fat, "Средний жир")?;
        opt_percentage_f64(&self.avg_milk_protein, "Средний белок")?;
        opt_non_negative_f64(&self.avg_weight, "Средний вес")?;
        opt_positive_i32(&self.max_robot_feed_types, "Макс. типов корма робота")?;
        opt_non_negative_f64(&self.max_feed_intake_robot, "Макс. потребление робота")?;
        opt_non_negative_f64(&self.min_feed_intake_robot, "Мин. потребление робота")?;
        opt_positive_i32(&self.number_of_cows, "Количество коров")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct FeedFilter {
    pub animal_id: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
