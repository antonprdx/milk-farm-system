use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use super::{BirthRemarkType, GenderType};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Calving {
    pub id: i32,
    pub animal_id: i32,
    pub calving_date: NaiveDate,
    pub remarks: Option<String>,
    pub lac_number: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Calf {
    pub id: i32,
    pub calving_id: i32,
    pub life_number: Option<String>,
    pub gender: GenderType,
    pub birth_remark: Option<BirthRemarkType>,
    pub keep: Option<bool>,
    pub weight: Option<f64>,
    pub born_dead: Option<bool>,
    pub animal_number: Option<i64>,
    pub calf_name: Option<String>,
    pub hair_color_code: Option<String>,
    pub born_dead_reason_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Insemination {
    pub id: i32,
    pub animal_id: i32,
    pub insemination_date: NaiveDate,
    pub sire_code: Option<String>,
    pub insemination_type: Option<String>,
    pub charge_number: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Pregnancy {
    pub id: i32,
    pub animal_id: i32,
    pub pregnancy_date: NaiveDate,
    pub pregnancy_type: Option<String>,
    pub insemination_date: Option<NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Heat {
    pub id: i32,
    pub animal_id: i32,
    pub heat_date: NaiveDate,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DryOff {
    pub id: i32,
    pub animal_id: i32,
    pub dry_off_date: NaiveDate,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCalving {
    pub animal_id: i32,
    pub calving_date: NaiveDate,
    pub remarks: Option<String>,
    pub lac_number: Option<i32>,
    pub calves: Option<Vec<CreateCalf>>,
}

impl CreateCalving {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        positive_i32(self.animal_id, "ID животного")?;
        date_not_future(&self.calving_date, "Дата отёла")?;
        opt_max_len(&self.remarks, 500, "Примечания")?;
        if let Some(n) = self.lac_number
            && n < 0
        {
            return Err(crate::errors::AppError::BadRequest(
                "Номер лактации не может быть отрицательным".into(),
            ));
        }
        if let Some(ref calves) = self.calves {
            if calves.len() > 5 {
                return Err(crate::errors::AppError::BadRequest(
                    "Максимум 5 телят на один отёл".into(),
                ));
            }
            for calf in calves {
                calf.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCalf {
    pub life_number: Option<String>,
    pub gender: GenderType,
    pub birth_remark: Option<BirthRemarkType>,
    pub keep: Option<bool>,
    pub weight: Option<f64>,
    pub born_dead: Option<bool>,
    pub animal_number: Option<i64>,
    pub calf_name: Option<String>,
    pub hair_color_code: Option<String>,
    pub born_dead_reason_id: Option<i32>,
}

impl CreateCalf {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        opt_max_len(&self.life_number, 50, "Номер жизни телёнка")?;
        opt_non_negative_f64(&self.weight, "Вес телёнка")?;
        opt_max_len(&self.calf_name, 100, "Кличка телёнка")?;
        opt_max_len(&self.hair_color_code, 20, "Код масти телёнка")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateInsemination {
    pub animal_id: i32,
    pub insemination_date: NaiveDate,
    pub sire_code: Option<String>,
    pub insemination_type: Option<String>,
    pub charge_number: Option<String>,
}

impl CreateInsemination {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        positive_i32(self.animal_id, "ID животного")?;
        date_not_future(&self.insemination_date, "Дата осеменения")?;
        opt_max_len(&self.sire_code, 50, "Код быка")?;
        opt_max_len(&self.insemination_type, 50, "Тип осеменения")?;
        opt_max_len(&self.charge_number, 50, "Номер партии")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePregnancy {
    pub animal_id: i32,
    pub pregnancy_date: NaiveDate,
    pub pregnancy_type: Option<String>,
    pub insemination_date: Option<NaiveDate>,
}

impl CreatePregnancy {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        positive_i32(self.animal_id, "ID животного")?;
        date_not_future(&self.pregnancy_date, "Дата стельности")?;
        opt_date_not_future(&self.insemination_date, "Дата осеменения")?;
        opt_max_len(&self.pregnancy_type, 50, "Тип стельности")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateHeat {
    pub animal_id: i32,
    pub heat_date: NaiveDate,
}

impl CreateHeat {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        positive_i32(self.animal_id, "ID животного")?;
        date_not_future(&self.heat_date, "Дата охоты")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDryOff {
    pub animal_id: i32,
    pub dry_off_date: NaiveDate,
}

impl CreateDryOff {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        positive_i32(self.animal_id, "ID животного")?;
        date_not_future(&self.dry_off_date, "Дата запуска")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCalving {
    pub calving_date: Option<NaiveDate>,
    pub remarks: Option<String>,
    pub lac_number: Option<i32>,
}

impl UpdateCalving {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref d) = self.calving_date {
            date_not_future(d, "Дата отёла")?;
        }
        opt_max_len(&self.remarks, 500, "Примечания")?;
        if let Some(n) = self.lac_number
            && n < 0
        {
            return Err(crate::errors::AppError::BadRequest(
                "Номер лактации не может быть отрицательным".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateInsemination {
    pub insemination_date: Option<NaiveDate>,
    pub sire_code: Option<String>,
    pub insemination_type: Option<String>,
    pub charge_number: Option<String>,
}

impl UpdateInsemination {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref d) = self.insemination_date {
            date_not_future(d, "Дата осеменения")?;
        }
        opt_max_len(&self.sire_code, 50, "Код быка")?;
        opt_max_len(&self.insemination_type, 50, "Тип осеменения")?;
        opt_max_len(&self.charge_number, 50, "Номер партии")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdatePregnancy {
    pub pregnancy_date: Option<NaiveDate>,
    pub pregnancy_type: Option<String>,
    pub insemination_date: Option<NaiveDate>,
}

impl UpdatePregnancy {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref d) = self.pregnancy_date {
            date_not_future(d, "Дата стельности")?;
        }
        if let Some(ref d) = self.insemination_date {
            date_not_future(d, "Дата осеменения")?;
        }
        opt_max_len(&self.pregnancy_type, 50, "Тип стельности")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateHeat {
    pub heat_date: Option<NaiveDate>,
}

impl UpdateHeat {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref d) = self.heat_date {
            date_not_future(d, "Дата охоты")?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateDryOff {
    pub dry_off_date: Option<NaiveDate>,
}

impl UpdateDryOff {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref d) = self.dry_off_date {
            date_not_future(d, "Дата запуска")?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct ReproductionFilter {
    pub animal_id: Option<i32>,
    pub life_number: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
