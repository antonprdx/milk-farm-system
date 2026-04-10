use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BulkTankTest {
    pub id: i32,
    pub date: NaiveDate,
    pub fat: f64,
    pub protein: f64,
    pub lactose: Option<f64>,
    pub scc: Option<i32>,
    pub ffa: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBulkTankTest {
    pub date: NaiveDate,
    pub fat: f64,
    pub protein: f64,
    pub lactose: Option<f64>,
    pub scc: Option<i32>,
    pub ffa: Option<f64>,
}

impl CreateBulkTankTest {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        date_not_future(&self.date, "Дата")?;
        positive_percentage_f64(self.fat, "Жир")?;
        positive_percentage_f64(self.protein, "Белок")?;
        opt_percentage_f64(&self.lactose, "Лактоза")?;
        if let Some(s) = self.scc {
            if s < 0 {
                return Err(crate::errors::AppError::BadRequest(
                    "СОК не может быть отрицательным".into(),
                ));
            }
        }
        opt_non_negative_f64(&self.ffa, "FFA")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateBulkTankTest {
    pub date: Option<NaiveDate>,
    pub fat: Option<f64>,
    pub protein: Option<f64>,
    pub lactose: Option<Option<f64>>,
    pub scc: Option<Option<i32>>,
    pub ffa: Option<Option<f64>>,
}

impl UpdateBulkTankTest {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        opt_date_not_future(&self.date, "Дата")?;
        if let Some(v) = self.fat {
            positive_percentage_f64(v, "Жир")?;
        }
        if let Some(v) = self.protein {
            positive_percentage_f64(v, "Белок")?;
        }
        if let Some(Some(v)) = self.lactose {
            percentage_f64(v, "Лактоза")?;
        }
        if let Some(Some(s)) = self.scc {
            if s < 0 {
                return Err(crate::errors::AppError::BadRequest(
                    "СОК не может быть отрицательным".into(),
                ));
            }
        }
        if let Some(Some(v)) = self.ffa {
            non_negative_f64(v, "FFA")?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct BulkTankFilter {
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
