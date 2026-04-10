use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contact {
    pub id: i32,
    pub name: String,
    pub contact_type_id: Option<i32>,
    pub contact_type_name: Option<String>,
    pub farm_number: Option<String>,
    pub phone_cell: Option<String>,
    pub phone_home: Option<String>,
    pub phone_work: Option<String>,
    pub email: Option<String>,
    pub company_name: Option<String>,
    pub description: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContact {
    pub name: String,
    pub type_id: Option<i32>,
    pub farm_number: Option<String>,
    pub active: bool,
    pub phone_cell: Option<String>,
    pub phone_home: Option<String>,
    pub phone_work: Option<String>,
    pub email: Option<String>,
    pub company_name: Option<String>,
    pub description: Option<String>,
}

impl CreateContact {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        required_non_empty(&self.name, "Имя")?;
        max_len(&self.name, 200, "Имя")?;
        opt_positive_i32(&self.type_id, "Тип контакта")?;
        opt_max_len(&self.farm_number, 50, "Номер фермы")?;
        opt_max_len(&self.phone_cell, 30, "Мобильный телефон")?;
        opt_max_len(&self.phone_home, 30, "Домашний телефон")?;
        opt_max_len(&self.phone_work, 30, "Рабочий телефон")?;
        opt_email(&self.email)?;
        opt_max_len(&self.email, 200, "Email")?;
        opt_max_len(&self.company_name, 200, "Компания")?;
        opt_max_len(&self.description, 500, "Описание")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateContact {
    pub name: Option<String>,
    pub type_id: Option<i32>,
    pub farm_number: Option<String>,
    pub active: Option<bool>,
    pub phone_cell: Option<String>,
    pub phone_home: Option<String>,
    pub phone_work: Option<String>,
    pub email: Option<String>,
    pub company_name: Option<String>,
    pub description: Option<String>,
}

impl UpdateContact {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref n) = self.name {
            required_non_empty(n, "Имя")?;
            max_len(n, 200, "Имя")?;
        }
        opt_positive_i32(&self.type_id, "Тип контакта")?;
        opt_max_len(&self.farm_number, 50, "Номер фермы")?;
        opt_max_len(&self.phone_cell, 30, "Мобильный телефон")?;
        opt_max_len(&self.phone_home, 30, "Домашний телефон")?;
        opt_max_len(&self.phone_work, 30, "Рабочий телефон")?;
        opt_email(&self.email)?;
        opt_max_len(&self.email, 200, "Email")?;
        opt_max_len(&self.company_name, 200, "Компания")?;
        opt_max_len(&self.description, 500, "Описание")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct ContactFilter {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
