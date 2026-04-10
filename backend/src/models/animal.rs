use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use super::GenderType;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Animal {
    pub id: i32,
    pub life_number: Option<String>,
    pub name: Option<String>,
    pub user_number: Option<i64>,
    pub gender: GenderType,
    pub birth_date: NaiveDate,
    pub hair_color_code: Option<String>,
    pub father_life_number: Option<String>,
    pub mother_life_number: Option<String>,
    pub description: Option<String>,
    pub ucn_number: Option<String>,
    pub use_as_sire: Option<bool>,
    pub location: Option<String>,
    pub group_number: Option<i32>,
    pub keep: Option<bool>,
    pub gestation: Option<i32>,
    pub responder_number: Option<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAnimal {
    pub life_number: Option<String>,
    pub name: Option<String>,
    pub user_number: Option<i64>,
    pub gender: GenderType,
    pub birth_date: NaiveDate,
    pub hair_color_code: Option<String>,
    pub father_life_number: Option<String>,
    pub mother_life_number: Option<String>,
    pub description: Option<String>,
    pub ucn_number: Option<String>,
    pub use_as_sire: Option<bool>,
    pub location: Option<String>,
    pub group_number: Option<i32>,
    pub keep: Option<bool>,
    pub gestation: Option<i32>,
    pub responder_number: Option<String>,
}

impl CreateAnimal {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        date_not_future(&self.birth_date, "Дата рождения")?;
        opt_max_len(&self.life_number, 50, "Номер жизни")?;
        opt_max_len(&self.name, 100, "Кличка")?;
        if let Some(n) = self.user_number {
            if n < 0 {
                return Err(crate::errors::AppError::BadRequest(
                    "Пользовательский номер не может быть отрицательным".into(),
                ));
            }
        }
        opt_max_len(&self.hair_color_code, 20, "Код масти")?;
        opt_max_len(&self.father_life_number, 50, "Номер жизни отца")?;
        opt_max_len(&self.mother_life_number, 50, "Номер жизни матери")?;
        opt_max_len(&self.description, 500, "Описание")?;
        opt_max_len(&self.ucn_number, 50, "UCN номер")?;
        opt_max_len(&self.location, 100, "Локация")?;
        opt_max_len(&self.responder_number, 50, "Номер респондера")?;
        if let Some(g) = self.gestation {
            if g < 0 || g > 500 {
                return Err(crate::errors::AppError::BadRequest(
                    "Срок стельности должен быть от 0 до 500 дней".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateAnimal {
    pub name: Option<String>,
    pub hair_color_code: Option<String>,
    pub description: Option<String>,
    pub ucn_number: Option<String>,
    pub use_as_sire: Option<bool>,
    pub location: Option<String>,
    pub group_number: Option<i32>,
    pub keep: Option<bool>,
    pub gestation: Option<i32>,
    pub responder_number: Option<String>,
    pub active: Option<bool>,
}

impl UpdateAnimal {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        opt_max_len(&self.name, 100, "Кличка")?;
        opt_max_len(&self.hair_color_code, 20, "Код масти")?;
        opt_max_len(&self.description, 500, "Описание")?;
        opt_max_len(&self.ucn_number, 50, "UCN номер")?;
        opt_max_len(&self.location, 100, "Локация")?;
        opt_max_len(&self.responder_number, 50, "Номер респондера")?;
        if let Some(g) = self.gestation {
            if g < 0 || g > 500 {
                return Err(crate::errors::AppError::BadRequest(
                    "Срок стельности должен быть от 0 до 500 дней".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct AnimalFilter {
    pub life_number: Option<String>,
    pub ucn_number: Option<String>,
    pub name: Option<String>,
    pub active: Option<bool>,
    pub gender: Option<GenderType>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
