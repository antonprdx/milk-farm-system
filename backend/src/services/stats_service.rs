use serde::Serialize;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::services::retry::retry_db;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_animals: i64,
    pub total_females: i64,
    pub milk_today: f64,
    pub in_heat: i64,
    pub pregnant: i64,
}

pub async fn dashboard_stats(pool: &PgPool) -> Result<DashboardStats, AppError> {
    let pool = pool.clone();
    retry_db(move || {
        let pool = pool.clone();
        async move {
            let (r1, r2, r3, r4, r5) = tokio::try_join!(
                async {
                    let (v,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM animals WHERE active = true")
                        .fetch_one(&pool).await.map_err(AppError::Database)?;
                    Ok::<_, AppError>(v)
                },
                async {
                    let (v,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM animals WHERE active = true AND gender = 'female'")
                        .fetch_one(&pool).await.map_err(AppError::Database)?;
                    Ok::<_, AppError>(v)
                },
                async {
                    let (v,): (Option<f64>,) = sqlx::query_as("SELECT SUM(milk_amount) FROM milk_day_productions WHERE date = CURRENT_DATE")
                        .fetch_one(&pool).await.map_err(AppError::Database)?;
                    Ok::<_, AppError>(v)
                },
                async {
                    let (v,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM heats WHERE heat_date = CURRENT_DATE")
                        .fetch_one(&pool).await.map_err(AppError::Database)?;
                    Ok::<_, AppError>(v)
                },
                async {
                    let (v,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pregnancies WHERE pregnancy_date = CURRENT_DATE")
                        .fetch_one(&pool).await.map_err(AppError::Database)?;
                    Ok::<_, AppError>(v)
                },
            )?;

            Ok::<_, AppError>(DashboardStats {
                total_animals: r1,
                total_females: r2,
                milk_today: r3.unwrap_or(0.0),
                in_heat: r4,
                pregnant: r5,
            })
        }
    }).await
}
