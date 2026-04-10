use serde::Serialize;
use sqlx::PgPool;

use crate::errors::AppError;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_animals: i64,
    pub total_females: i64,
    pub milk_today: f64,
    pub in_heat: i64,
    pub pregnant: i64,
}

pub async fn dashboard_stats(pool: &PgPool) -> Result<DashboardStats, AppError> {
    let (total_animals,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM animals WHERE active = true",
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (total_females,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM animals WHERE active = true AND gender = 'female'",
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (milk_today,): (Option<f64>,) = sqlx::query_as(
        "SELECT SUM(milk_amount) FROM milk_day_productions WHERE date = CURRENT_DATE",
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (in_heat,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM heats WHERE heat_date = CURRENT_DATE",
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (pregnant,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pregnancies WHERE pregnancy_date = CURRENT_DATE",
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(DashboardStats {
        total_animals,
        total_females,
        milk_today: milk_today.unwrap_or(0.0),
        in_heat,
        pregnant,
    })
}
