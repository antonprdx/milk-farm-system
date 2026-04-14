use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Sire {
    pub id: i32,
    pub sire_code: Option<String>,
    pub life_number: Option<String>,
    pub name: Option<String>,
    pub active: Option<bool>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSire {
    pub sire_code: Option<String>,
    pub life_number: Option<String>,
    pub name: Option<String>,
    pub active: Option<bool>,
}

impl CreateSire {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        opt_max_len(&self.sire_code, 100, "Код быка")?;
        opt_max_len(&self.life_number, 100, "Жизненный номер")?;
        opt_max_len(&self.name, 200, "Имя")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateSire {
    pub sire_code: Option<String>,
    pub life_number: Option<String>,
    pub name: Option<String>,
    pub active: Option<bool>,
}

impl UpdateSire {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        opt_max_len(&self.sire_code, 100, "Код быка")?;
        opt_max_len(&self.life_number, 100, "Жизненный номер")?;
        opt_max_len(&self.name, 200, "Имя")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct SireFilter {
    pub search: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
