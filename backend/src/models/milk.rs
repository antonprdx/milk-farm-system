use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MilkDayProduction {
    pub id: i32,
    pub animal_id: i32,
    pub date: NaiveDate,
    pub milk_amount: Option<f64>,
    pub avg_amount: Option<f64>,
    pub avg_weight: Option<f64>,
    pub isk: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MilkVisit {
    pub id: i32,
    pub animal_id: i32,
    pub visit_datetime: chrono::DateTime<chrono::Utc>,
    pub milk_amount: Option<f64>,
    pub duration_seconds: Option<i32>,
    pub milk_destination: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MilkQuality {
    pub id: i32,
    pub animal_id: i32,
    pub date: NaiveDate,
    pub milk_amount: Option<f64>,
    pub avg_amount: Option<f64>,
    pub avg_weight: Option<f64>,
    pub isk: Option<f64>,
    pub fat_percentage: Option<f64>,
    pub protein_percentage: Option<f64>,
    pub lactose_percentage: Option<f64>,
    pub scc: Option<i32>,
    pub milkings: Option<i32>,
    pub refusals: Option<i32>,
    pub failures: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMilkDayProduction {
    pub animal_id: i32,
    pub date: NaiveDate,
    pub milk_amount: Option<f64>,
    pub avg_amount: Option<f64>,
    pub avg_weight: Option<f64>,
    pub isk: Option<f64>,
}

impl CreateMilkDayProduction {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        positive_i32(self.animal_id, "ID животного")?;
        date_not_future(&self.date, "Дата")?;
        opt_non_negative_f64(&self.milk_amount, "Надой")?;
        opt_non_negative_f64(&self.avg_amount, "Средний надой")?;
        opt_non_negative_f64(&self.avg_weight, "Средний вес")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateMilkDayProduction {
    pub date: Option<NaiveDate>,
    pub milk_amount: Option<f64>,
    pub avg_amount: Option<f64>,
    pub avg_weight: Option<f64>,
    pub isk: Option<f64>,
}

impl UpdateMilkDayProduction {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref d) = self.date {
            date_not_future(d, "Дата")?;
        }
        opt_non_negative_f64(&self.milk_amount, "Надой")?;
        opt_non_negative_f64(&self.avg_amount, "Средний надой")?;
        opt_non_negative_f64(&self.avg_weight, "Средний вес")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct MilkFilter {
    pub animal_id: Option<i32>,
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
