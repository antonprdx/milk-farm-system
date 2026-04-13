use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Location {
    pub id: i32,
    pub name: String,
    pub location_type: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateLocation {
    pub name: String,
    pub location_type: Option<String>,
}

impl CreateLocation {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        required_non_empty(&self.name, "Название")?;
        max_len(&self.name, 200, "Название")?;
        opt_max_len(&self.location_type, 100, "Тип локации")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateLocation {
    pub name: Option<String>,
    pub location_type: Option<String>,
}

impl UpdateLocation {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref n) = self.name {
            required_non_empty(n, "Название")?;
            max_len(n, 200, "Название")?;
        }
        opt_max_len(&self.location_type, 100, "Тип локации")?;
        Ok(())
    }
}
