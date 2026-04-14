use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Transfer {
    pub id: i32,
    pub animal_id: i32,
    pub transfer_date: chrono::DateTime<chrono::Utc>,
    pub transfer_type: String,
    pub reason_id: Option<i32>,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTransfer {
    pub animal_id: i32,
    pub transfer_date: chrono::DateTime<chrono::Utc>,
    pub transfer_type: String,
    pub reason_id: Option<i32>,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
}

impl CreateTransfer {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        positive_i32(self.animal_id, "ID животного")?;
        required_non_empty(&self.transfer_type, "Тип перемещения")?;
        max_len(&self.transfer_type, 100, "Тип перемещения")?;
        opt_max_len(&self.from_location, 200, "Откуда")?;
        opt_max_len(&self.to_location, 200, "Куда")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTransfer {
    pub transfer_date: Option<chrono::DateTime<chrono::Utc>>,
    pub transfer_type: Option<String>,
    pub reason_id: Option<i32>,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
}

impl UpdateTransfer {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref t) = self.transfer_type {
            required_non_empty(t, "Тип перемещения")?;
            max_len(t, 100, "Тип перемещения")?;
        }
        opt_max_len(&self.from_location, 200, "Откуда")?;
        opt_max_len(&self.to_location, 200, "Куда")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TransferFilter {
    pub animal_id: Option<String>,
    pub transfer_type: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
