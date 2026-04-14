use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct VetRecord {
    pub id: i32,
    pub animal_id: i32,
    pub record_type: VetRecordType,
    pub status: VetRecordStatus,
    pub event_date: NaiveDate,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub medication: Option<String>,
    pub dosage: Option<String>,
    pub withdrawal_days: Option<i32>,
    pub withdrawal_end_date: Option<NaiveDate>,
    pub veterinarian: Option<String>,
    pub notes: Option<String>,
    pub follow_up_date: Option<NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "vet_record_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VetRecordType {
    Vaccination,
    Treatment,
    Disease,
    Surgery,
    Deworming,
    HoofCare,
    Examination,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "vet_record_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VetRecordStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateVetRecord {
    pub animal_id: i32,
    pub record_type: VetRecordType,
    pub status: Option<VetRecordStatus>,
    pub event_date: NaiveDate,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub medication: Option<String>,
    pub dosage: Option<String>,
    pub withdrawal_days: Option<i32>,
    pub veterinarian: Option<String>,
    pub notes: Option<String>,
    pub follow_up_date: Option<NaiveDate>,
}

impl CreateVetRecord {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        date_not_future(&self.event_date, "Дата события")?;
        opt_max_len(&self.diagnosis, 500, "Диагноз")?;
        opt_max_len(&self.treatment, 1000, "Лечение")?;
        opt_max_len(&self.medication, 200, "Препарат")?;
        opt_max_len(&self.dosage, 100, "Дозировка")?;
        opt_max_len(&self.veterinarian, 100, "Ветеринар")?;
        opt_max_len(&self.notes, 1000, "Заметки")?;
        if let Some(d) = self.withdrawal_days && d < 0 {
            return Err(crate::errors::AppError::BadRequest(
                "Срок ожидания не может быть отрицательным".into(),
            ));
        }
        opt_date_not_future(&self.follow_up_date, "Дата повторного осмотра")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateVetRecord {
    pub record_type: Option<VetRecordType>,
    pub status: Option<VetRecordStatus>,
    pub event_date: Option<NaiveDate>,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub medication: Option<String>,
    pub dosage: Option<String>,
    pub withdrawal_days: Option<i32>,
    pub veterinarian: Option<String>,
    pub notes: Option<String>,
    pub follow_up_date: Option<NaiveDate>,
}

impl UpdateVetRecord {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(d) = self.event_date {
            date_not_future(&d, "Дата события")?;
        }
        opt_max_len(&self.diagnosis, 500, "Диагноз")?;
        opt_max_len(&self.treatment, 1000, "Лечение")?;
        opt_max_len(&self.medication, 200, "Препарат")?;
        opt_max_len(&self.dosage, 100, "Дозировка")?;
        opt_max_len(&self.veterinarian, 100, "Ветеринар")?;
        opt_max_len(&self.notes, 1000, "Заметки")?;
        if let Some(d) = self.withdrawal_days && d < 0 {
            return Err(crate::errors::AppError::BadRequest(
                "Срок ожидания не может быть отрицательным".into(),
            ));
        }
        opt_date_not_future(&self.follow_up_date, "Дата повторного осмотра")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct VetRecordFilter {
    pub animal_id: Option<i32>,
    pub record_type: Option<VetRecordType>,
    pub status: Option<VetRecordStatus>,
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct WeightRecord {
    pub id: i32,
    pub animal_id: i32,
    pub weight_kg: f64,
    pub bcs: Option<f64>,
    pub measure_date: NaiveDate,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateWeightRecord {
    pub animal_id: i32,
    pub weight_kg: f64,
    pub bcs: Option<f64>,
    pub measure_date: NaiveDate,
    pub notes: Option<String>,
}

impl CreateWeightRecord {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if self.weight_kg <= 0.0 {
            return Err(crate::errors::AppError::BadRequest(
                "Вес должен быть положительным числом".into(),
            ));
        }
        if let Some(bcs) = self.bcs && !(1.0..=5.0).contains(&bcs) {
            return Err(crate::errors::AppError::BadRequest(
                "BCS должен быть от 1.0 до 5.0".into(),
            ));
        }
        date_not_future(&self.measure_date, "Дата измерения")?;
        opt_max_len(&self.notes, 500, "Заметки")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct WeightRecordFilter {
    pub animal_id: Option<i32>,
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
