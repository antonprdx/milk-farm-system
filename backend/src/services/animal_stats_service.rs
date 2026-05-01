use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::animal_stats::{
    AnimalStats, LatestMetrics, MilkDataPoint, ReproductionSummary, SccDataPoint,
};

pub async fn get_animal_stats(pool: &PgPool, animal_id: i32) -> Result<AnimalStats, AppError> {
    let animal_ref_date: Option<(chrono::NaiveDate,)> = sqlx::query_as(
        "SELECT MAX(date) FROM milk_day_productions WHERE animal_id = $1",
    )
    .bind(animal_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let ref_date_expr = match &animal_ref_date {
        Some((d,)) => format!("'{}'::date", d),
        None => "get_ref_date()".to_string(),
    };

    let milk_sql = format!(
        "SELECT date, COALESCE(milk_amount, 0) AS amount
         FROM milk_day_productions
         WHERE animal_id = $1 AND date >= {ref_date_expr} - INTERVAL '30 days'
         ORDER BY date"
    );
    let milk_30d = sqlx::query_as::<_, MilkDataPoint>(&milk_sql)
        .bind(animal_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

    let scc_sql = format!(
        "SELECT date, COALESCE(scc, 0) AS scc
         FROM milk_quality
         WHERE animal_id = $1 AND date >= {ref_date_expr} - INTERVAL '90 days'
         ORDER BY date"
    );
    let scc_90d = sqlx::query_as::<_, SccDataPoint>(&scc_sql)
        .bind(animal_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

    let metrics_sql = format!(
        "SELECT AVG(milk_amount)::double precision,
                (SELECT scc FROM milk_quality WHERE animal_id = $1 ORDER BY date DESC LIMIT 1),
                AVG(avg_weight)::double precision,
                AVG(isk)::double precision
         FROM milk_day_productions
         WHERE animal_id = $1 AND date >= {ref_date_expr} - INTERVAL '30 days'"
    );
    let metrics_row: (Option<f64>, Option<i32>, Option<f64>, Option<f64>) = sqlx::query_as(&metrics_sql)
        .bind(animal_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

    let latest_metrics = LatestMetrics {
        avg_milk_30d: metrics_row.0,
        last_scc: metrics_row.1,
        avg_weight_30d: metrics_row.2,
        avg_isk_30d: metrics_row.3,
    };

    let last_calving: Option<(chrono::NaiveDate, Option<i32>)> = sqlx::query_as(
        "SELECT calving_date, lac_number FROM calvings
         WHERE animal_id = $1 ORDER BY calving_date DESC LIMIT 1",
    )
    .bind(animal_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let total_inseminations: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM inseminations WHERE animal_id = $1")
            .bind(animal_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

    let latest_pregnancy: Option<(chrono::NaiveDate,)> = sqlx::query_as(
        "SELECT pregnancy_date FROM pregnancies
         WHERE animal_id = $1 ORDER BY pregnancy_date DESC LIMIT 1",
    )
    .bind(animal_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let is_pregnant = latest_pregnancy.is_some();

    let expected_calving = if let Some((calving_date, _)) = &last_calving {
        let last_insem: Option<(chrono::NaiveDate,)> = sqlx::query_as(
            "SELECT insemination_date FROM inseminations
             WHERE animal_id = $1 AND insemination_date > $2
             ORDER BY insemination_date DESC LIMIT 1",
        )
        .bind(animal_id)
        .bind(calving_date)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        last_insem.map(|(d,)| d + chrono::Duration::days(283))
    } else {
        let last_insem: Option<(chrono::NaiveDate,)> = sqlx::query_as(
            "SELECT insemination_date FROM inseminations
             WHERE animal_id = $1 ORDER BY insemination_date DESC LIMIT 1",
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        last_insem.map(|(d,)| d + chrono::Duration::days(283))
    };

    let latest_dry_off: Option<(chrono::NaiveDate,)> = sqlx::query_as(
        "SELECT dry_off_date FROM dry_offs
         WHERE animal_id = $1 ORDER BY dry_off_date DESC LIMIT 1",
    )
    .bind(animal_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    let is_dry = if let Some((dry_date,)) = &latest_dry_off {
        if let Some((calving_date, _)) = &last_calving {
            dry_date >= calving_date
        } else {
            true
        }
    } else {
        false
    };

    let ref_date_for_dim = match &animal_ref_date {
        Some((d,)) => *d,
        None => chrono::Local::now().date_naive(),
    };

    let days_in_milk = if let Some((calving_date, _)) = &last_calving {
        Some((ref_date_for_dim - *calving_date).num_days())
    } else {
        None
    };

    let reproduction = ReproductionSummary {
        last_calving_date: last_calving.as_ref().map(|(d, _)| *d),
        total_inseminations: total_inseminations.0,
        expected_calving_date: expected_calving,
        is_pregnant,
        lactation_number: last_calving.as_ref().and_then(|(_, lac)| *lac),
        days_in_milk,
        is_dry,
    };

    Ok(AnimalStats {
        milk_production_30d: milk_30d,
        scc_trend_90d: scc_90d,
        latest_metrics,
        reproduction,
    })
}
