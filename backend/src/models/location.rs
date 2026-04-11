use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Location {
    pub id: i32,
    pub name: String,
    pub location_type: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
