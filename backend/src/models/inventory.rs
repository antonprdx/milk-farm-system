use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct InventoryItem {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub unit: String,
    pub quantity: f64,
    pub min_quantity: f64,
    pub cost_per_unit: Option<f64>,
    pub supplier: Option<String>,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateInventoryItem {
    pub name: String,
    pub category: String,
    pub unit: Option<String>,
    pub quantity: Option<f64>,
    pub min_quantity: Option<f64>,
    pub cost_per_unit: Option<f64>,
    pub supplier: Option<String>,
    pub notes: Option<String>,
}

impl CreateInventoryItem {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        required_non_empty(&self.name, "Название")?;
        max_len(&self.name, 200, "Название")?;
        required_non_empty(&self.category, "Категория")?;
        max_len(&self.category, 50, "Категория")?;
        opt_max_len(&self.unit, 20, "Единица")?;
        opt_non_negative_f64(&self.quantity, "Количество")?;
        opt_non_negative_f64(&self.min_quantity, "Мин. количество")?;
        opt_non_negative_f64(&self.cost_per_unit, "Цена за единицу")?;
        opt_max_len(&self.supplier, 200, "Поставщик")?;
        opt_max_len(&self.notes, 500, "Заметки")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateInventoryItem {
    pub name: Option<String>,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub min_quantity: Option<f64>,
    pub cost_per_unit: Option<f64>,
    pub supplier: Option<String>,
    pub notes: Option<String>,
}

impl UpdateInventoryItem {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        if let Some(ref v) = self.name {
            required_non_empty(v, "Название")?;
            max_len(v, 200, "Название")?;
        }
        if let Some(ref v) = self.category {
            required_non_empty(v, "Категория")?;
            max_len(v, 50, "Категория")?;
        }
        opt_max_len(&self.unit, 20, "Единица")?;
        opt_non_negative_f64(&self.min_quantity, "Мин. количество")?;
        opt_non_negative_f64(&self.cost_per_unit, "Цена за единицу")?;
        opt_max_len(&self.supplier, 200, "Поставщик")?;
        opt_max_len(&self.notes, 500, "Заметки")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct InventoryFilter {
    pub category: Option<String>,
    pub low_stock: Option<bool>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct InventoryTransaction {
    pub id: i32,
    pub item_id: i32,
    pub transaction_type: String,
    pub quantity: f64,
    pub notes: Option<String>,
    pub transaction_date: NaiveDate,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateInventoryTransaction {
    pub item_id: i32,
    pub transaction_type: String,
    pub quantity: f64,
    pub notes: Option<String>,
    pub transaction_date: Option<NaiveDate>,
}

impl CreateInventoryTransaction {
    pub fn validate(&self) -> Result<(), AppError> {
        use crate::validation::*;
        positive_i32(self.item_id, "ID позиции")?;
        if self.quantity <= 0.0 {
            return Err(AppError::BadRequest(
                "Количество должно быть больше нуля".into(),
            ));
        }
        if self.transaction_type != "in"
            && self.transaction_type != "out"
            && self.transaction_type != "adjustment"
        {
            return Err(AppError::BadRequest(
                "Тип транзакции должен быть in, out или adjustment".into(),
            ));
        }
        opt_max_len(&self.notes, 500, "Заметки")?;
        Ok(())
    }
}
