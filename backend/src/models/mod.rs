pub mod analytics;
pub mod animal;
pub mod bulk_tank;
pub mod contact;
pub mod feed;
pub mod fitness;
pub mod grazing;
pub mod location;
pub mod milk;
pub mod paginated_result;
pub mod pagination;
pub mod reproduction;
pub mod user;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "gender_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GenderType {
    Male,
    Female,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "birth_remark_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BirthRemarkType {
    Normal,
    AbnormalCalf,
    AlivePrematureBorn,
    Abortion,
    TwinCalfFreeMartin,
    TwinCalfSameSex,
    DepartedAutoTransferOut,
}
