pub mod analytics;
pub mod animal;
pub mod animal_stats;
pub mod bulk_tank;
pub mod contact;
pub mod feed;
pub mod fitness;
pub mod grazing;
pub mod location;
pub mod milk;
pub mod pagination;
pub mod preferences;
pub mod reports;
pub mod reproduction;
pub mod sire;
pub mod system_settings;
pub mod task;
pub mod timeline;
pub mod transfer;
pub mod user;
pub mod vet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "gender_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GenderType {
    Male,
    Female,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
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
