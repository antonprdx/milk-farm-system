use chrono::NaiveDate;
use serde::Serialize;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::reports::*;

#[derive(Debug, sqlx::FromRow)]
struct HerdOverviewDbRow {
    date: String,
    cow_count: i64,
    total_milk: Option<f64>,
    avg_day_production: Option<f64>,
    total_milkings: Option<i64>,
    total_refusals: Option<i64>,
    total_failures: Option<i64>,
    milk_separated: Option<i64>,
    avg_scc: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct RestFeedDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    feed_date: String,
    feed_number: i32,
    total_planned: f64,
    rest_feed: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
struct CowDailyProductionDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    date: String,
    milk_amount: Option<f64>,
    avg_amount: Option<f64>,
    avg_weight: Option<f64>,
    isk: Option<f64>,
    scc: Option<i32>,
    fat_pct: Option<f64>,
    protein_pct: Option<f64>,
    lactose_pct: Option<f64>,
    feed_total: Option<f64>,
    feed_rest: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
struct RobotPerformanceDbRow {
    device_address: Option<i32>,
    date: String,
    avg_milk_speed: Option<f64>,
    max_milk_speed: Option<f64>,
    milkings: i64,
    avg_lf_milk_time: Option<f64>,
    avg_lr_milk_time: Option<f64>,
    avg_rf_milk_time: Option<f64>,
    avg_rr_milk_time: Option<f64>,
    avg_lf_dead_milk_time: Option<f64>,
    avg_lr_dead_milk_time: Option<f64>,
    avg_rf_dead_milk_time: Option<f64>,
    avg_rr_dead_milk_time: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct FailedMilkingDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    visit_datetime: String,
    device_address: Option<i32>,
    milk_yield: Option<f64>,
    lf_colour: Option<String>,
    lr_colour: Option<String>,
    rf_colour: Option<String>,
    rr_colour: Option<String>,
    lf_conductivity: Option<i32>,
    lr_conductivity: Option<i32>,
    rf_conductivity: Option<i32>,
    rr_conductivity: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
struct UdderHealthDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    visit_datetime: String,
    lf_conductivity: Option<i32>,
    lr_conductivity: Option<i32>,
    rf_conductivity: Option<i32>,
    rr_conductivity: Option<i32>,
    lf_colour: Option<String>,
    lr_colour: Option<String>,
    rf_colour: Option<String>,
    rr_colour: Option<String>,
    latest_scc: Option<i32>,
    milk_yield: Option<f64>,
    deviation_day_prod: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct MilkDayProductionTimeDbRow {
    date: String,
    total_milk: Option<f64>,
    avg_milk_per_cow: Option<f64>,
    cow_count: i64,
    milkings: Option<i64>,
    refusals: Option<i64>,
    failures: Option<i64>,
    avg_weight: Option<f64>,
    total_feed: Option<f64>,
    total_rest_feed: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct VisitBehaviorDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    total_milkings: i64,
    total_refusals: i64,
    avg_milk_per_milking: Option<f64>,
    avg_duration_seconds: Option<f64>,
    last_visit: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct CalendarCalvingDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    lac_number: Option<i32>,
    group_number: Option<i32>,
    last_insemination_date: Option<String>,
    expected_calving_date: Option<String>,
    days_until_calving: Option<i64>,
    sire_code: Option<String>,
    days_pregnant: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct CalendarHeatDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    last_heat_date: Option<String>,
    expected_heat_date: Option<String>,
    days_until_heat: Option<i64>,
    days_in_lactation: Option<i64>,
    inseminated: bool,
    overdue: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct CalendarPregnancyCheckDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    insemination_date: Option<String>,
    sire_code: Option<String>,
    days_since_insemination: Option<i64>,
    pregnancy_confirmed: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct HealthActivityDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    health_index: Option<f64>,
    activity_deviation: Option<f64>,
    rumination_minutes: Option<i32>,
    max_rumination_change_24h: Option<i32>,
    rumination_3day_diff: Option<i32>,
    latest_milk: Option<f64>,
    avg_milk_7d: Option<f64>,
    milk_deviation_pct: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct CowRobotEfficiencyDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    milk_per_box_time_week: Option<f64>,
    avg_milk_speed: Option<f64>,
    avg_treatment_time: Option<f64>,
    avg_milking_time: Option<f64>,
    milkings_7d: i64,
    total_milk_7d: Option<f64>,
    avg_milk_per_milking: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct LactationAnalysisDbRow {
    lac_number: i32,
    dim: i32,
    avg_milk: Option<f64>,
    avg_visits: Option<f64>,
    avg_feed: Option<f64>,
    avg_weight: Option<f64>,
    avg_fat: Option<f64>,
    avg_protein: Option<f64>,
    cow_count: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct FeedPerTypeDayDbRow {
    date: String,
    feed_type: String,
    feed_type_name: String,
    total_amount_product: Option<f64>,
    total_amount_dm: Option<f64>,
    total_cost: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct FeedPerCowDayDbRow {
    date: String,
    animal_count: i64,
    avg_total_per_cow: Option<f64>,
    avg_concentrate_per_cow: Option<f64>,
    avg_roughage_per_cow: Option<f64>,
    avg_cost_per_cow: Option<f64>,
    avg_rumination_minutes: Option<f64>,
    avg_day_production: Option<f64>,
    avg_lactation_days: Option<f64>,
    feed_efficiency: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct TransitionDbRow {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    days_relative: i64,
    milk_24h: Option<f64>,
    sick_chance: Option<f64>,
    rumination_3day_diff: Option<i32>,
    rumination_minutes: Option<i32>,
    feed_total: Option<f64>,
    feed_rest: Option<i32>,
    latest_scc: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
struct ExportMilkDbRow {
    animal_name: String,
    date: String,
    milk_amount: Option<f64>,
    avg_amount: Option<f64>,
    avg_weight: Option<f64>,
    isk: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct MilkSummary {
    pub total_milk: f64,
    pub count_days: i64,
    pub avg_per_animal: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ReproductionSummary {
    pub total_calvings: i64,
    pub total_inseminations: i64,
    pub total_pregnancies: i64,
    pub total_heats: i64,
    pub total_dry_offs: i64,
}

#[derive(Debug, Serialize)]
pub struct FeedSummary {
    pub total_feed_kg: f64,
    pub total_visits: i64,
}

pub async fn milk_summary(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<MilkSummary, AppError> {
    let (total_milk, count_days, avg_per_animal) = tokio::try_join!(
        async {
            let (v,): (f64,) = sqlx::query_as(
                "SELECT COALESCE(SUM(milk_amount), 0) FROM milk_day_productions WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)"
            )
            .bind(from_date)
            .bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
        async {
            let (v,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM milk_day_productions WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)"
            )
            .bind(from_date)
            .bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
        async {
            let (v,): (Option<f64>,) = sqlx::query_as(
                "SELECT AVG(milk_amount) FROM milk_day_productions WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)"
            )
            .bind(from_date)
            .bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
    )?;

    Ok(MilkSummary {
        total_milk,
        count_days,
        avg_per_animal,
    })
}

pub async fn reproduction_summary(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<ReproductionSummary, AppError> {
    let (total_calvings, total_inseminations, total_pregnancies, total_heats, total_dry_offs) = tokio::try_join!(
        async {
            let (v,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM calvings WHERE ($1::date IS NULL OR calving_date >= $1) AND ($2::date IS NULL OR calving_date <= $2)"
            )
            .bind(from_date).bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
        async {
            let (v,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM inseminations WHERE ($1::date IS NULL OR insemination_date >= $1) AND ($2::date IS NULL OR insemination_date <= $2)"
            )
            .bind(from_date).bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
        async {
            let (v,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM pregnancies WHERE ($1::date IS NULL OR pregnancy_date >= $1) AND ($2::date IS NULL OR pregnancy_date <= $2)"
            )
            .bind(from_date).bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
        async {
            let (v,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM heats WHERE ($1::date IS NULL OR heat_date >= $1) AND ($2::date IS NULL OR heat_date <= $2)"
            )
            .bind(from_date).bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
        async {
            let (v,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM dry_offs WHERE ($1::date IS NULL OR dry_off_date >= $1) AND ($2::date IS NULL OR dry_off_date <= $2)"
            )
            .bind(from_date).bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
    )?;

    Ok(ReproductionSummary {
        total_calvings,
        total_inseminations,
        total_pregnancies,
        total_heats,
        total_dry_offs,
    })
}

pub async fn feed_summary(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<FeedSummary, AppError> {
    let (total_feed, total_visits) = tokio::try_join!(
        async {
            let (v,): (f64,) = sqlx::query_as(
                "SELECT COALESCE(SUM(total), 0) FROM feed_day_amounts WHERE ($1::date IS NULL OR feed_date >= $1) AND ($2::date IS NULL OR feed_date <= $2)"
            )
            .bind(from_date).bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
        async {
            let (v,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM feed_visits WHERE ($1::date IS NULL OR visit_datetime::date >= $1) AND ($2::date IS NULL OR visit_datetime::date <= $2)"
            )
            .bind(from_date).bind(till_date)
            .fetch_one(pool).await.map_err(AppError::Database)?;
            Ok::<_, AppError>(v)
        },
    )?;

    Ok(FeedSummary {
        total_feed_kg: total_feed,
        total_visits,
    })
}

pub async fn herd_overview(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<HerdOverviewResponse, AppError> {
    let rows: Vec<HerdOverviewDbRow> = sqlx::query_as(
        "SELECT d.date::text,
                COALESCE(d.cow_count, 0),
                d.total_milk,
                d.avg_milk as avg_day_production,
                mq.milkings as total_milkings,
                mq.refusals as total_refusals,
                mq.failures as total_failures,
                sep.cnt as milk_separated,
                mq.avg_scc
         FROM (
             SELECT date,
                    COUNT(DISTINCT animal_id)::int8 as cow_count,
                    SUM(milk_amount)::float8 as total_milk,
                    AVG(milk_amount)::float8 as avg_milk
             FROM milk_day_productions
             WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)
             GROUP BY date
         ) d
         LEFT JOIN (
             SELECT date,
                    SUM(milkings)::int8 as milkings,
                    SUM(refusals)::int8 as refusals,
                    SUM(failures)::int8 as failures,
                    AVG(scc)::float8 as avg_scc
             FROM milk_quality
             WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)
             GROUP BY date
         ) mq ON mq.date = d.date
         LEFT JOIN (
             SELECT visit_datetime::date as vd, COUNT(*)::int8 as cnt
             FROM milk_visits
             WHERE milk_destination = 2
               AND ($1::date IS NULL OR visit_datetime::date >= $1)
               AND ($2::date IS NULL OR visit_datetime::date <= $2)
             GROUP BY visit_datetime::date
         ) sep ON sep.vd = d.date
         ORDER BY d.date",
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let period: Vec<HerdOverviewRow> = rows
        .into_iter()
        .map(|row| {
            HerdOverviewRow {
                date: row.date,
                cow_count: row.cow_count,
                total_milk: row.total_milk,
                avg_day_production: row.avg_day_production,
                total_milkings: row.total_milkings,
                total_refusals: row.total_refusals,
                total_failures: row.total_failures,
                milk_separated: row.milk_separated,
                avg_scc: row.avg_scc,
            }
        })
        .collect();

    let n = period.len().max(1) as f64;
    let avg_cow_count = period.iter().map(|r| r.cow_count as f64).sum::<f64>() / n;
    let avg_milk = period.iter().filter_map(|r| r.total_milk).partial_avg();
    let avg_milkings = period
        .iter()
        .filter_map(|r| r.total_milkings.map(|v| v as f64))
        .partial_avg();
    let avg_failures = period
        .iter()
        .filter_map(|r| r.total_failures.map(|v| v as f64))
        .partial_avg();
    let avg_scc = period.iter().filter_map(|r| r.avg_scc).partial_avg();

    Ok(HerdOverviewResponse {
        period,
        avg_cow_count: (avg_cow_count * 100.0).round() / 100.0,
        avg_milk,
        avg_milkings,
        avg_failures,
        avg_scc,
    })
}

pub async fn rest_feed_report(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<RestFeedResponse, AppError> {
    let rows: Vec<RestFeedDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number, f.feed_date::text as feed_date, f.feed_number, f.total as total_planned, f.rest_feed
         FROM feed_day_amounts f
         JOIN animals a ON a.id = f.animal_id
         WHERE f.rest_feed IS NOT NULL AND f.rest_feed > 0
           AND ($1::date IS NULL OR f.feed_date >= $1) AND ($2::date IS NULL OR f.feed_date <= $2)
         ORDER BY f.rest_feed DESC, f.feed_date DESC",
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let rest_rows: Vec<RestFeedRow> = rows
        .into_iter()
        .map(|row| {
            let rest_feed_pct =
                row.rest_feed.map(|rf| (rf as f64 / row.total_planned * 100.0 * 100.0).round() / 100.0);
            RestFeedRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                feed_date: row.feed_date,
                feed_number: row.feed_number,
                total_planned: row.total_planned,
                rest_feed: row.rest_feed,
                rest_feed_pct,
            }
        })
        .collect();

    let total_planned_all: Option<f64> = sqlx::query_scalar(
        "SELECT SUM(total) FROM feed_day_amounts WHERE ($1::date IS NULL OR feed_date >= $1) AND ($2::date IS NULL OR feed_date <= $2)"
    )
    .bind(from_date).bind(till_date)
    .fetch_optional(pool).await.map_err(AppError::Database)?
    .flatten();

    let total_rest: Option<f64> = sqlx::query_scalar(
        "SELECT SUM(rest_feed::float8) FROM feed_day_amounts WHERE rest_feed IS NOT NULL AND ($1::date IS NULL OR feed_date >= $1) AND ($2::date IS NULL OR feed_date <= $2)"
    )
    .bind(from_date).bind(till_date)
    .fetch_optional(pool).await.map_err(AppError::Database)?
    .flatten();

    let total_rest_feed_pct = match (total_rest, total_planned_all) {
        (Some(rest), Some(planned)) if planned > 0.0 => {
            Some((rest / planned * 100.0 * 100.0).round() / 100.0)
        }
        _ => None,
    };

    Ok(RestFeedResponse {
        rows: rest_rows,
        total_rest_feed_pct,
    })
}

pub async fn cow_daily_production(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
    animal_id: Option<i32>,
) -> Result<Vec<CowDailyProductionRow>, AppError> {
    let rows: Vec<CowDailyProductionDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number, md.date::text,
                md.milk_amount, md.avg_amount, md.avg_weight, md.isk,
                mq.scc, mq.fat_percentage as fat_pct, mq.protein_percentage as protein_pct, mq.lactose_percentage as lactose_pct,
                fd_agg.total as feed_total, fd_agg.rest as feed_rest
         FROM milk_day_productions md
         JOIN animals a ON a.id = md.animal_id
         LEFT JOIN milk_quality mq ON mq.animal_id = md.animal_id AND mq.date = md.date
         LEFT JOIN LATERAL (
             SELECT SUM(total)::float8 as total, SUM(COALESCE(rest_feed, 0))::int as rest
             FROM feed_day_amounts WHERE animal_id = md.animal_id AND feed_date = md.date
         ) fd_agg ON true
         WHERE ($1::date IS NULL OR md.date >= $1) AND ($2::date IS NULL OR md.date <= $2)
           AND ($3::int IS NULL OR md.animal_id = $3)
         ORDER BY md.date DESC, a.name",
    )
    .bind(from_date)
    .bind(till_date)
    .bind(animal_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            CowDailyProductionRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                date: row.date,
                milk_amount: row.milk_amount,
                avg_amount: row.avg_amount,
                avg_weight: row.avg_weight,
                isk: row.isk,
                scc: row.scc,
                fat_pct: row.fat_pct,
                protein_pct: row.protein_pct,
                lactose_pct: row.lactose_pct,
                feed_total: row.feed_total,
                feed_rest: row.feed_rest,
            }
        })
        .collect())
}

pub async fn robot_performance(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<Vec<RobotPerformanceRow>, AppError> {
    let rows: Vec<RobotPerformanceDbRow> = sqlx::query_as(
        "SELECT r.device_address, r.milking_date::date::text as date,
                AVG(r.milk_speed)::float8 as avg_milk_speed, MAX(r.milk_speed_max)::float8 as max_milk_speed,
                COUNT(*)::int8 as milkings,
                AVG(r.lf_milk_time)::float8 as avg_lf_milk_time, AVG(r.lr_milk_time)::float8 as avg_lr_milk_time,
                AVG(r.rf_milk_time)::float8 as avg_rf_milk_time, AVG(r.rr_milk_time)::float8 as avg_rr_milk_time,
                AVG(r.lf_dead_milk_time)::float8 as avg_lf_dead_milk_time, AVG(r.lr_dead_milk_time)::float8 as avg_lr_dead_milk_time,
                AVG(r.rf_dead_milk_time)::float8 as avg_rf_dead_milk_time, AVG(r.rr_dead_milk_time)::float8 as avg_rr_dead_milk_time
         FROM robot_milk_data r
         WHERE ($1::date IS NULL OR r.milking_date::date >= $1) AND ($2::date IS NULL OR r.milking_date::date <= $2)
         GROUP BY r.device_address, r.milking_date::date
         ORDER BY date DESC, r.device_address"
    )
    .bind(from_date).bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            RobotPerformanceRow {
                device_address: row.device_address,
                date: row.date,
                avg_milk_speed: row.avg_milk_speed,
                max_milk_speed: row.max_milk_speed,
                milkings: row.milkings,
                avg_lf_milk_time: row.avg_lf_milk_time,
                avg_lr_milk_time: row.avg_lr_milk_time,
                avg_rf_milk_time: row.avg_rf_milk_time,
                avg_rr_milk_time: row.avg_rr_milk_time,
                avg_lf_dead_milk_time: row.avg_lf_dead_milk_time,
                avg_lr_dead_milk_time: row.avg_lr_dead_milk_time,
                avg_rf_dead_milk_time: row.avg_rf_dead_milk_time,
                avg_rr_dead_milk_time: row.avg_rr_dead_milk_time,
            }
        })
        .collect())
}

pub async fn failed_milkings(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<Vec<FailedMilkingRow>, AppError> {
    let rows: Vec<FailedMilkingDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number, v.visit_datetime::text,
                v.device_address, v.milk_yield,
                v.lf_colour_code as lf_colour, v.lr_colour_code as lr_colour, v.rf_colour_code as rf_colour, v.rr_colour_code as rr_colour,
                v.lf_conductivity, v.lr_conductivity, v.rf_conductivity, v.rr_conductivity
         FROM milk_visit_quality v
         JOIN animals a ON a.id = v.animal_id
         WHERE v.success_milking = false
           AND ($1::date IS NULL OR v.visit_datetime::date >= $1) AND ($2::date IS NULL OR v.visit_datetime::date <= $2)
           AND NOT EXISTS (
               SELECT 1 FROM milk_visit_quality v2
               WHERE v2.animal_id = v.animal_id
                 AND v2.visit_datetime > v.visit_datetime
                 AND v2.success_milking = true
                 AND v2.visit_datetime < v.visit_datetime + INTERVAL '24 hours'
           )
         ORDER BY v.visit_datetime DESC"
    )
    .bind(from_date).bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            FailedMilkingRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                visit_datetime: row.visit_datetime,
                device_address: row.device_address,
                milk_yield: row.milk_yield,
                lf_colour: row.lf_colour,
                lr_colour: row.lr_colour,
                rf_colour: row.rf_colour,
                rr_colour: row.rr_colour,
                lf_conductivity: row.lf_conductivity,
                lr_conductivity: row.lr_conductivity,
                rf_conductivity: row.rf_conductivity,
                rr_conductivity: row.rr_conductivity,
            }
        })
        .collect())
}

pub async fn udder_health_worklist(pool: &PgPool) -> Result<UdderHealthResponse, AppError> {
    udder_health_query(pool, UdderHealthParams {
        interval: "24 hours",
        cond_threshold: 83,
        check_deviation: true,
    }).await
}

pub async fn udder_health_analyze(pool: &PgPool) -> Result<UdderHealthResponse, AppError> {
    udder_health_query(pool, UdderHealthParams {
        interval: "14 days",
        cond_threshold: 80,
        check_deviation: false,
    }).await
}

struct UdderHealthParams<'a> {
    interval: &'a str,
    cond_threshold: i32,
    check_deviation: bool,
}

async fn udder_health_query(pool: &PgPool, params: UdderHealthParams<'_>) -> Result<UdderHealthResponse, AppError> {
    let threshold = params.cond_threshold;
    let deviation_clause = if params.check_deviation { "OR deviation.dev < -3.0" } else { "" };
    let sql = format!(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number, v.visit_datetime::text,
                v.lf_conductivity, v.lr_conductivity, v.rf_conductivity, v.rr_conductivity,
                v.lf_colour_code as lf_colour, v.lr_colour_code as lr_colour, v.rf_colour_code as rf_colour, v.rr_colour_code as rr_colour,
                latest_scc.scc as latest_scc, v.milk_yield, deviation.dev as deviation_day_prod
         FROM milk_visit_quality v
         JOIN animals a ON a.id = v.animal_id
         LEFT JOIN LATERAL (
             SELECT scc FROM milk_quality WHERE animal_id = a.id ORDER BY date DESC LIMIT 1
         ) latest_scc ON true
         LEFT JOIN LATERAL (
             SELECT (short.avg - long.avg)::float8 as dev
             FROM (SELECT AVG(milk_amount)::float8 as avg FROM milk_day_productions WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '2 days') short,
                  (SELECT AVG(milk_amount)::float8 as avg FROM milk_day_productions WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '14 days' AND date < CURRENT_DATE - INTERVAL '2 days') long
         ) deviation ON true
         WHERE v.visit_datetime >= NOW() - INTERVAL '{interval}'
           AND (
               COALESCE(v.lf_conductivity, 0) > {threshold} OR COALESCE(v.lr_conductivity, 0) > {threshold}
               OR COALESCE(v.rf_conductivity, 0) > {threshold} OR COALESCE(v.rr_conductivity, 0) > {threshold}
               OR v.lf_colour_code IS NOT NULL OR v.lr_colour_code IS NOT NULL
               OR v.rf_colour_code IS NOT NULL OR v.rr_colour_code IS NOT NULL
               {deviation_clause}
           )
         ORDER BY v.visit_datetime DESC",
        interval = params.interval,
        threshold = threshold,
        deviation_clause = deviation_clause,
    );
    let rows: Vec<UdderHealthDbRow> = sqlx::query_as(&sql)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let result: Vec<UdderHealthRow> = rows
        .into_iter()
        .map(|row| {
            let mut attention_quarters = Vec::new();
            let quarters = [
                ("LF", row.lf_conductivity, &row.lf_colour),
                ("LR", row.lr_conductivity, &row.lr_colour),
                ("RF", row.rf_conductivity, &row.rf_colour),
                ("RR", row.rr_conductivity, &row.rr_colour),
            ];
            for (name, cond, col) in &quarters {
                let mut reasons = Vec::new();
                if let Some(c) = cond
                    && *c > threshold
                {
                    reasons.push(format!("cond={}", c));
                }
                if let Some(cl) = col
                    && !cl.is_empty()
                {
                    reasons.push(format!("color={}", cl));
                }
                if !reasons.is_empty() {
                    attention_quarters.push(format!("{}: {}", name, reasons.join(", ")));
                }
            }
            UdderHealthRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                visit_datetime: row.visit_datetime,
                lf_conductivity: row.lf_conductivity,
                lr_conductivity: row.lr_conductivity,
                rf_conductivity: row.rf_conductivity,
                rr_conductivity: row.rr_conductivity,
                lf_colour: row.lf_colour,
                lr_colour: row.lr_colour,
                rf_colour: row.rf_colour,
                rr_colour: row.rr_colour,
                latest_scc: row.latest_scc,
                milk_yield: row.milk_yield,
                deviation_day_prod: row.deviation_day_prod,
                attention_quarters,
                separation: None,
            }
        })
        .collect();

    Ok(UdderHealthResponse { rows: result })
}

pub async fn milk_day_production_time(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<Vec<MilkDayProductionTimeRow>, AppError> {
    let rows: Vec<MilkDayProductionTimeDbRow> = sqlx::query_as(
        "SELECT d.date::text, day_milk.total as total_milk, day_milk.avg_per_cow, day_milk.cnt as cow_count,
                mq_sum.milkings, mq_sum.refusals, mq_sum.failures,
                day_milk.avg_weight, feed_sum.total_feed, feed_sum.rest_feed as total_rest_feed
         FROM (SELECT DISTINCT date FROM milk_day_productions
               WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)) d
         LEFT JOIN LATERAL (
             SELECT SUM(milk_amount)::float8 as total, AVG(milk_amount)::float8 as avg_per_cow,
                    COUNT(DISTINCT animal_id)::int8 as cnt, AVG(avg_weight)::float8 as avg_weight
             FROM milk_day_productions WHERE date = d.date
         ) day_milk ON true
         LEFT JOIN LATERAL (
             SELECT SUM(milkings)::int8 as milkings, SUM(refusals)::int8 as refusals, SUM(failures)::int8 as failures
             FROM milk_quality WHERE date = d.date
         ) mq_sum ON true
         LEFT JOIN LATERAL (
             SELECT SUM(total)::float8 as total_feed, SUM(COALESCE(rest_feed, 0))::int8 as rest_feed
             FROM feed_day_amounts WHERE feed_date = d.date
         ) feed_sum ON true
         ORDER BY d.date"
    )
    .bind(from_date).bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            MilkDayProductionTimeRow {
                date: row.date,
                total_milk: row.total_milk,
                avg_milk_per_cow: row.avg_milk_per_cow,
                cow_count: row.cow_count,
                milkings: row.milkings,
                refusals: row.refusals,
                failures: row.failures,
                avg_weight: row.avg_weight,
                total_feed: row.total_feed,
                total_rest_feed: row.total_rest_feed,
            }
        })
        .collect())
}

pub async fn visit_behavior(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<Vec<VisitBehaviorRow>, AppError> {
    let rows: Vec<VisitBehaviorDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number,
                COALESCE(milk_cnt.cnt, 0) as total_milkings,
                COALESCE(refusal_cnt.cnt, 0) as total_refusals,
                milk_avg.avg_milk as avg_milk_per_milking,
                dur_avg.avg_dur as avg_duration_seconds,
                last_v.last_visit::text as last_visit
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT COUNT(*)::int8 as cnt FROM milk_visits v
             WHERE v.animal_id = a.id AND v.milk_amount > 0
               AND ($1::date IS NULL OR v.visit_datetime::date >= $1) AND ($2::date IS NULL OR v.visit_datetime::date <= $2)
         ) milk_cnt ON true
         LEFT JOIN LATERAL (
             SELECT COUNT(*)::int8 as cnt FROM milk_quality q
             WHERE q.animal_id = a.id AND q.refusals > 0
               AND ($1::date IS NULL OR q.date >= $1) AND ($2::date IS NULL OR q.date <= $2)
         ) refusal_cnt ON true
         LEFT JOIN LATERAL (
             SELECT AVG(milk_amount)::float8 as avg_milk FROM milk_visits v
             WHERE v.animal_id = a.id AND v.milk_amount > 0
               AND ($1::date IS NULL OR v.visit_datetime::date >= $1) AND ($2::date IS NULL OR v.visit_datetime::date <= $2)
         ) milk_avg ON true
         LEFT JOIN LATERAL (
             SELECT AVG(duration_seconds)::float8 as avg_dur FROM milk_visits v
             WHERE v.animal_id = a.id AND v.duration_seconds IS NOT NULL
               AND ($1::date IS NULL OR v.visit_datetime::date >= $1) AND ($2::date IS NULL OR v.visit_datetime::date <= $2)
         ) dur_avg ON true
         LEFT JOIN LATERAL (
             SELECT MAX(visit_datetime)::text as last_visit FROM milk_visits v WHERE v.animal_id = a.id
         ) last_v ON true
         WHERE a.active = true AND a.gender = 'female'
         ORDER BY milk_avg.avg_milk ASC NULLS LAST
         LIMIT 200"
    )
    .bind(from_date).bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            VisitBehaviorRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                total_milkings: row.total_milkings,
                total_refusals: row.total_refusals,
                avg_milk_per_milking: row.avg_milk_per_milking,
                avg_duration_seconds: row.avg_duration_seconds,
                milk_frequency_setting: None,
                last_visit: row.last_visit,
            }
        })
        .collect())
}

pub async fn calendar(pool: &PgPool) -> Result<CalendarResponse, AppError> {
    let today = chrono::Utc::now().date_naive();

    let calvings_data: Vec<CalendarCalvingDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number, latest_c.lac_number, a.group_number,
                latest_ins.ins_date::text as last_insemination_date, latest_ins.expected_calving::text as expected_calving_date,
                latest_ins.days_left as days_until_calving, latest_ins.sire_code, latest_ins.days_pregnant
         FROM animals a
         CROSS JOIN LATERAL (
             SELECT c.lac_number FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
         ) latest_c
         CROSS JOIN LATERAL (
             SELECT i.insemination_date::text as ins_date,
                    (i.insemination_date + 283)::date::text as expected_calving,
                    (i.insemination_date + 283 - CURRENT_DATE)::int8 as days_left,
                    i.sire_code,
                    (CURRENT_DATE - i.insemination_date)::int8 as days_pregnant
             FROM inseminations i
             WHERE i.animal_id = a.id
               AND NOT EXISTS (SELECT 1 FROM calvings c2 WHERE c2.animal_id = i.animal_id AND c2.calving_date > i.insemination_date)
               AND NOT EXISTS (SELECT 1 FROM dry_offs d WHERE d.animal_id = i.animal_id AND d.dry_off_date > i.insemination_date)
             ORDER BY i.insemination_date DESC LIMIT 1
         ) latest_ins
         WHERE a.active = true AND a.gender = 'female'
           AND NOT EXISTS (SELECT 1 FROM calvings c3 WHERE c3.animal_id = a.id AND c3.calving_date >= CURRENT_DATE - INTERVAL '30 days')
         ORDER BY latest_ins.days_left"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let expected_calvings: Vec<CalendarCalvingRow> = calvings_data
        .into_iter()
        .map(|row| {
            CalendarCalvingRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                lac_number: row.lac_number,
                group_number: row.group_number,
                last_insemination_date: row.last_insemination_date,
                expected_calving_date: row.expected_calving_date,
                days_until_calving: row.days_until_calving,
                sire_code: row.sire_code,
                days_pregnant: row.days_pregnant,
            }
        })
        .collect();

    let expected_dry_offs: Vec<CalendarDryOffRow> = expected_calvings
        .iter()
        .filter_map(|c| {
            let exp_str = c.expected_calving_date.as_deref()?;
            let exp = chrono::NaiveDate::parse_from_str(exp_str, "%Y-%m-%d").ok()?;
            let rec = exp - chrono::Duration::days(60);
            let days_until = (rec - today).num_days();
            Some(CalendarDryOffRow {
                animal_id: c.animal_id,
                animal_name: c.animal_name.clone(),
                life_number: c.life_number.clone(),
                expected_calving_date: c.expected_calving_date.clone(),
                recommended_dry_off_date: Some(rec.format("%Y-%m-%d").to_string()),
                days_until_dry_off: Some(days_until),
                lac_number: c.lac_number,
            })
        })
        .filter(|d| d.days_until_dry_off.is_some_and(|dl| dl <= 30))
        .collect();

    let heats_data: Vec<CalendarHeatDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number,
                last_h.heat_date::text as last_heat_date, (last_h.heat_date + 21)::date::text as expected_heat_date,
                (last_h.heat_date + 21 - CURRENT_DATE)::int8 as days_until_heat,
                days_lac.days_in_lac as days_in_lactation,
                COALESCE(has_ins.has, false) as inseminated,
                COALESCE((last_h.heat_date + 21) < CURRENT_DATE, false) as overdue
         FROM animals a
         CROSS JOIN LATERAL (
             SELECT heat_date FROM heats WHERE animal_id = a.id ORDER BY heat_date DESC LIMIT 1
         ) last_h
         LEFT JOIN LATERAL (
             SELECT days_in_lac FROM (
                 SELECT (CURRENT_DATE - calving_date)::int8 as days_in_lac
                 FROM calvings WHERE animal_id = a.id ORDER BY calving_date DESC LIMIT 1
             ) sub
         ) days_lac ON true
         LEFT JOIN LATERAL (
             SELECT true as has FROM inseminations WHERE animal_id = a.id AND insemination_date >= last_h.heat_date LIMIT 1
         ) has_ins ON true
         WHERE a.active = true AND a.gender = 'female'
           AND NOT EXISTS (SELECT 1 FROM pregnancies p WHERE p.animal_id = a.id AND p.pregnancy_date >= last_h.heat_date)
         ORDER BY days_until_heat"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let expected_heats: Vec<CalendarHeatRow> = heats_data
        .into_iter()
        .map(|row| {
            CalendarHeatRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                last_heat_date: row.last_heat_date,
                expected_heat_date: row.expected_heat_date,
                days_until_heat: row.days_until_heat,
                days_in_lactation: row.days_in_lactation,
                inseminated: row.inseminated,
                overdue: row.overdue,
            }
        })
        .collect();

    let preg_checks: Vec<CalendarPregnancyCheckDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number,
                latest_i.ins_date::text as insemination_date, latest_i.sire_code,
                (CURRENT_DATE - latest_i.ins_date)::int8 as days_since_insemination,
                COALESCE(has_preg.confirmed, false) as pregnancy_confirmed
         FROM animals a
         CROSS JOIN LATERAL (
             SELECT insemination_date as ins_date, sire_code
             FROM inseminations WHERE animal_id = a.id ORDER BY insemination_date DESC LIMIT 1
         ) latest_i
         LEFT JOIN LATERAL (
             SELECT true as confirmed FROM pregnancies WHERE animal_id = a.id AND pregnancy_date >= latest_i.ins_date LIMIT 1
         ) has_preg ON true
         WHERE a.active = true AND a.gender = 'female'
           AND NOT has_preg.confirmed
           AND (CURRENT_DATE - latest_i.ins_date) BETWEEN 28 AND 60
         ORDER BY (CURRENT_DATE - latest_i.ins_date) DESC"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let pregnancy_checks: Vec<CalendarPregnancyCheckRow> = preg_checks
        .into_iter()
        .map(|row| {
            CalendarPregnancyCheckRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                insemination_date: row.insemination_date,
                sire_code: row.sire_code,
                days_since_insemination: row.days_since_insemination,
                pregnancy_confirmed: row.pregnancy_confirmed,
            }
        })
        .collect();

    Ok(CalendarResponse {
        expected_calvings,
        expected_dry_offs,
        expected_heats,
        pregnancy_checks,
    })
}

pub async fn health_activity_rumination(pool: &PgPool) -> Result<Vec<HealthActivityRow>, AppError> {
    let rows: Vec<HealthActivityDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number,
                health_idx.idx as health_index,
                act_dev.deviation as activity_deviation,
                rum_recent.rum_minutes as rumination_minutes,
                rum_change.max_change as max_rumination_change_24h,
                rum_diff.diff_3d as rumination_3day_diff,
                latest_m.milk as latest_milk,
                milk_7d.avg_milk as avg_milk_7d,
                milk_dev.dev_pct as milk_deviation_pct
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT AVG(activity_counter)::float8 as idx FROM activities
             WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '1 day'
         ) act_recent ON true
         LEFT JOIN LATERAL (
             SELECT AVG(activity_counter)::float8 as baseline FROM activities
             WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '11 days' AND activity_datetime < CURRENT_DATE - INTERVAL '1 day'
         ) act_base ON true
         CROSS JOIN LATERAL (
             SELECT (act_recent.idx - act_base.baseline) as deviation
         ) act_dev
         CROSS JOIN LATERAL (
             SELECT act_recent.idx as idx
         ) health_idx
         LEFT JOIN LATERAL (
             SELECT rumination_minutes as rum_minutes FROM ruminations WHERE animal_id = a.id ORDER BY date DESC LIMIT 1
         ) rum_recent ON true
         LEFT JOIN LATERAL (
             SELECT (MAX(rumination_minutes) - MIN(rumination_minutes))::int as max_change
             FROM ruminations WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '1 day'
         ) rum_change ON true
         LEFT JOIN LATERAL (
             SELECT (r1.rumination_minutes - r2.rumination_minutes) as diff_3d
             FROM ruminations r1, LATERAL (
                 SELECT rumination_minutes FROM ruminations WHERE animal_id = a.id ORDER BY date DESC LIMIT 1 OFFSET 2
             ) r2
             WHERE r1.animal_id = a.id ORDER BY r1.date DESC LIMIT 1
         ) rum_diff ON true
         LEFT JOIN LATERAL (
             SELECT milk_amount as milk FROM milk_day_productions WHERE animal_id = a.id ORDER BY date DESC LIMIT 1
         ) latest_m ON true
         LEFT JOIN LATERAL (
             SELECT AVG(milk_amount)::float8 as avg_milk FROM milk_day_productions WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '7 days'
         ) milk_7d ON true
         CROSS JOIN LATERAL (
             SELECT CASE WHEN milk_7d.avg_milk > 0 THEN ((latest_m.milk - milk_7d.avg_milk) / milk_7d.avg_milk * 100.0) ELSE NULL END as dev_pct
         ) milk_dev
         WHERE a.active = true AND a.gender = 'female'
           AND (health_idx.idx < 90 OR rum_diff.diff_3d < -60 OR act_dev.deviation < -20)
         ORDER BY health_idx.idx ASC NULLS LAST
         LIMIT 100"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            HealthActivityRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                health_index: row.health_index,
                activity_deviation: row.activity_deviation,
                rumination_minutes: row.rumination_minutes,
                max_rumination_change_24h: row.max_rumination_change_24h,
                rumination_3day_diff: row.rumination_3day_diff,
                latest_milk: row.latest_milk,
                avg_milk_7d: row.avg_milk_7d,
                milk_deviation_pct: row.milk_deviation_pct,
            }
        })
        .collect())
}

pub async fn cow_robot_efficiency(pool: &PgPool) -> Result<Vec<CowRobotEfficiencyRow>, AppError> {
    let rows: Vec<CowRobotEfficiencyDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number,
                eff.milk_per_box_time as milk_per_box_time_week,
                rob.avg_speed as avg_milk_speed,
                eff.avg_treatment as avg_treatment_time,
                eff.avg_milking as avg_milking_time,
                v7d.visits as milkings_7d,
                v7d.total_milk as total_milk_7d,
                v7d.avg_per_milking as avg_milk_per_milking
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT SUM(v.milk_amount)::float8 / NULLIF(SUM(v.duration_seconds)::float8 / 60.0, 0) * (COUNT(*)::float8 / 7.0) as milk_per_box_time,
                    AVG(v.duration_seconds)::float8 / 60.0 as avg_treatment,
                    AVG(CASE WHEN v.milk_amount > 0 THEN v.duration_seconds END)::float8 / 60.0 as avg_milking
             FROM milk_visits v
             WHERE v.animal_id = a.id AND v.visit_datetime >= CURRENT_DATE - INTERVAL '7 days' AND v.duration_seconds > 0
         ) eff ON true
         LEFT JOIN LATERAL (
             SELECT AVG(r.milk_speed)::float8 as avg_speed FROM robot_milk_data r
             WHERE r.animal_id = a.id AND r.milking_date >= CURRENT_DATE - INTERVAL '7 days'
         ) rob ON true
         LEFT JOIN LATERAL (
             SELECT COUNT(*)::int8 as visits, SUM(milk_amount)::float8 as total_milk,
                    AVG(milk_amount)::float8 as avg_per_milking
             FROM milk_visits v
             WHERE v.animal_id = a.id AND v.milk_amount > 0 AND v.visit_datetime >= CURRENT_DATE - INTERVAL '7 days'
         ) v7d ON true
         WHERE a.active = true AND a.gender = 'female' AND v7d.visits > 0
         ORDER BY eff.milk_per_box_time ASC NULLS LAST
         LIMIT 100"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            CowRobotEfficiencyRow {
                animal_id: row.animal_id,
                animal_name: row.animal_name,
                life_number: row.life_number,
                milk_per_box_time_week: row.milk_per_box_time_week,
                avg_milk_speed: row.avg_milk_speed,
                avg_treatment_time: row.avg_treatment_time,
                avg_milking_time: row.avg_milking_time,
                milkings_7d: row.milkings_7d,
                total_milk_7d: row.total_milk_7d,
                avg_milk_per_milking: row.avg_milk_per_milking,
            }
        })
        .collect())
}

pub async fn lactation_analysis(
    pool: &PgPool,
    lac_number: Option<i32>,
) -> Result<Vec<LactationAnalysisResponse>, AppError> {
    let rows: Vec<LactationAnalysisDbRow> = sqlx::query_as(
        "SELECT c.lac_number,
                (md.date - c.calving_date)::int as dim,
                AVG(md.milk_amount)::float8 as avg_milk,
                AVG(vis.cnt)::float8 as avg_visits,
                AVG(fd.total)::float8 as avg_feed,
                AVG(md.avg_weight)::float8 as avg_weight,
                AVG(mq.fat_percentage)::float8 as avg_fat,
                AVG(mq.protein_percentage)::float8 as avg_protein,
                COUNT(DISTINCT md.animal_id)::int8 as cow_count
         FROM calvings c
         JOIN milk_day_productions md ON md.animal_id = c.animal_id AND md.date >= c.calving_date AND md.date < c.calving_date + INTERVAL '400 days'
         LEFT JOIN LATERAL (
             SELECT COUNT(*)::float8 as cnt FROM milk_visits v WHERE v.animal_id = md.animal_id AND v.visit_datetime::date = md.date
         ) vis ON true
         LEFT JOIN milk_quality mq ON mq.animal_id = md.animal_id AND mq.date = md.date
         LEFT JOIN feed_day_amounts fd ON fd.animal_id = md.animal_id AND fd.feed_date = md.date
         WHERE c.lac_number IS NOT NULL AND ($1::int IS NULL OR c.lac_number = $1)
         GROUP BY c.lac_number, (md.date - c.calving_date)::int
         ORDER BY c.lac_number, dim"
    )
    .bind(lac_number)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut map: std::collections::BTreeMap<i32, Vec<LactationAnalysisPoint>> =
        std::collections::BTreeMap::new();
    for row in rows {
        map.entry(row.lac_number).or_default().push(LactationAnalysisPoint {
            dim: row.dim,
            avg_milk: row.avg_milk,
            avg_visits: row.avg_visits,
            avg_feed: row.avg_feed,
            avg_weight: row.avg_weight,
            avg_fat: row.avg_fat,
            avg_protein: row.avg_protein,
            cow_count: row.cow_count,
        });
    }

    Ok(map
        .into_iter()
        .map(|(lac_number, points)| LactationAnalysisResponse { lac_number, points })
        .collect())
}

pub async fn feed_per_type_day(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<FeedPerTypeResponse, AppError> {
    let rows: Vec<FeedPerTypeDayDbRow> = sqlx::query_as(
        "SELECT fd.feed_date::text as date,
                ft.feed_type, ft.name as feed_type_name,
                SUM(fd.total)::float8 as total_amount_product,
                SUM(fd.total * ft.dry_matter_percentage / 100.0)::float8 as total_amount_dm,
                SUM(fd.total * ft.price)::float8 as total_cost
         FROM feed_day_amounts fd
         JOIN feed_types ft ON ft.number_of_feed_type = fd.feed_number
         WHERE ($1::date IS NULL OR fd.feed_date >= $1) AND ($2::date IS NULL OR fd.feed_date <= $2)
         GROUP BY fd.feed_date, ft.feed_type, ft.name
         ORDER BY fd.feed_date DESC, ft.feed_type",
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let milk_total: Option<f64> = sqlx::query_scalar(
        "SELECT SUM(milk_amount)::float8 FROM milk_day_productions WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)"
    )
    .bind(from_date).bind(till_date)
    .fetch_optional(pool).await.map_err(AppError::Database)?
    .flatten();

    let feed_rows: Vec<FeedPerTypeDayRow> = rows
        .into_iter()
        .map(|row| {
            let cost_per_100milk = match (row.total_cost, milk_total) {
                (Some(cost), Some(milk)) if milk > 0.0 => {
                    Some((cost / milk * 100.0 * 100.0).round() / 100.0)
                }
                _ => None,
            };
            FeedPerTypeDayRow {
                date: row.date,
                feed_type: row.feed_type,
                feed_type_name: row.feed_type_name,
                total_amount_product: row.total_amount_product,
                total_amount_dm: row.total_amount_dm,
                total_cost: row.total_cost,
                cost_per_100milk,
            }
        })
        .collect();

    let total_cost: Option<f64> = feed_rows.iter().filter_map(|r| r.total_cost).partial_sum();
    let avg_cost_per_100milk = match (total_cost, milk_total) {
        (Some(cost), Some(milk)) if milk > 0.0 => {
            Some((cost / milk * 100.0 * 100.0).round() / 100.0)
        }
        _ => None,
    };

    Ok(FeedPerTypeResponse {
        rows: feed_rows,
        avg_cost_per_100milk,
        total_cost,
    })
}

pub async fn feed_per_cow_day(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<Vec<FeedPerCowDayRow>, AppError> {
    let rows: Vec<FeedPerCowDayDbRow> = sqlx::query_as(
        "SELECT d.date::text,
                cow_cnt.cnt as animal_count,
                feed.avg_total as avg_total_per_cow,
                feed.avg_conc as avg_concentrate_per_cow,
                feed.avg_rough as avg_roughage_per_cow,
                feed.avg_cost as avg_cost_per_cow,
                rum.avg_rum as avg_rumination_minutes,
                milk.avg_milk as avg_day_production,
                lac.avg_lac_days as avg_lactation_days,
                CASE WHEN feed.avg_total > 0 THEN milk.avg_milk / feed.avg_total ELSE NULL END as feed_efficiency
         FROM (SELECT DISTINCT date FROM milk_day_productions
               WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)) d
         LEFT JOIN LATERAL (
             SELECT COUNT(DISTINCT animal_id)::int8 as cnt FROM milk_day_productions WHERE date = d.date
         ) cow_cnt ON true
         LEFT JOIN LATERAL (
             SELECT AVG(total)::float8 as avg_total,
                    AVG(CASE WHEN ft.feed_type = 'concentrate' THEN total ELSE 0 END)::float8 as avg_conc,
                    AVG(CASE WHEN ft.feed_type = 'roughage' THEN total ELSE 0 END)::float8 as avg_rough,
                    AVG(fd.total * ft.price)::float8 as avg_cost
             FROM feed_day_amounts fd
             JOIN feed_types ft ON ft.number_of_feed_type = fd.feed_number
             WHERE fd.feed_date = d.date
         ) feed ON true
         LEFT JOIN LATERAL (
             SELECT AVG(rumination_minutes)::float8 as avg_rum FROM ruminations WHERE date = d.date
         ) rum ON true
         LEFT JOIN LATERAL (
             SELECT AVG(milk_amount)::float8 as avg_milk FROM milk_day_productions WHERE date = d.date
         ) milk ON true
         LEFT JOIN LATERAL (
             SELECT AVG((d.date - c.calving_date)::float8) as avg_lac_days
             FROM calvings c
             JOIN milk_day_productions m ON m.animal_id = c.animal_id AND m.date = d.date AND c.calving_date <= d.date
             WHERE NOT EXISTS (SELECT 1 FROM calvings c2 WHERE c2.animal_id = c.animal_id AND c2.calving_date > c.calving_date AND c2.calving_date <= d.date)
         ) lac ON true
         ORDER BY d.date"
    )
    .bind(from_date).bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            FeedPerCowDayRow {
                date: row.date,
                animal_count: row.animal_count,
                avg_total_per_cow: row.avg_total_per_cow,
                avg_concentrate_per_cow: row.avg_concentrate_per_cow,
                avg_roughage_per_cow: row.avg_roughage_per_cow,
                avg_cost_per_cow: row.avg_cost_per_cow,
                avg_rumination_minutes: row.avg_rumination_minutes,
                avg_day_production: row.avg_day_production,
                avg_lactation_days: row.avg_lactation_days,
                feed_efficiency: row.feed_efficiency,
            }
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct HealthTaskRaw {
    animal_id: i32,
    animal_name: Option<String>,
    life_number: Option<String>,
    milk_drop_kg: Option<f64>,
    milk_deviation_pct: Option<f64>,
    cond_highest: Option<i32>,
    #[allow(dead_code)]
    cond_avg_14d: Option<f64>,
    scc: Option<i32>,
    act_deviation: Option<f64>,
    rum_diff_3d: Option<i32>,
    weight_trend: Option<f64>,
    total_weight_loss: Option<f64>,
    fat_protein_ratio: Option<f64>,
    feed_rest_pct: Option<f64>,
    highest_temp: Option<f64>,
    colour_atts: Option<String>,
    milk_trend_dev: Option<f64>,
    days_in_lactation: Option<i64>,
}

pub async fn health_task(pool: &PgPool) -> Result<HealthTaskResponse, AppError> {
    let rows: Vec<HealthTaskRaw> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number,
                milk_dev.drop_kg::float8 as milk_drop_kg,
                milk_dev.dev_pct::float8 as milk_deviation_pct,
                cond_highest.highest::int4 as cond_highest,
                cond_highest.avg_14d::float8 as cond_avg_14d,
                scc_latest.scc::int4 as scc,
                act_dev.deviation::float8 as act_deviation,
                rum_diff.diff_3d::int4 as rum_diff_3d,
                weight_trend.trend::float8 as weight_trend,
                weight_trend.total_loss::float8 as total_weight_loss,
                fpr.ratio::float8 as fat_protein_ratio,
                rest_pct.pct::float8 as feed_rest_pct,
                temp_highest.highest_temp::float8 as highest_temp,
                colour_atts.atts::text as colour_atts,
                milk_trend_dev.dev::float8 as milk_trend_dev,
                days_lac.days_in_lactation::int8 as days_in_lactation
         FROM animals a
         LEFT JOIN LATERAL (
             SELECT (long.avg - short.avg)::float8 as drop_kg,
                    CASE WHEN long.avg > 0 THEN ((long.avg - short.avg) / long.avg * 100.0) ELSE NULL END as dev_pct
             FROM (SELECT AVG(milk_amount)::float8 as avg FROM milk_day_productions WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '2 days') short,
                  (SELECT AVG(milk_amount)::float8 as avg FROM milk_day_productions WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '14 days' AND date < CURRENT_DATE - INTERVAL '2 days') long
         ) milk_dev ON true
         LEFT JOIN LATERAL (
             SELECT GREATEST(lf_conductivity, lr_conductivity, rf_conductivity, rr_conductivity) as highest,
                    NULL::float8 as avg_14d
             FROM milk_visit_quality WHERE animal_id = a.id ORDER BY visit_datetime DESC LIMIT 1
         ) cond_highest ON true
         LEFT JOIN LATERAL (
             SELECT scc FROM milk_quality WHERE animal_id = a.id ORDER BY date DESC LIMIT 1
         ) scc_latest ON true
         LEFT JOIN LATERAL (
             SELECT (recent.act - baseline.avg_act) as deviation FROM
                (SELECT AVG(activity_counter)::float8 as act FROM activities WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '1 day') recent,
                (SELECT AVG(activity_counter)::float8 as avg_act FROM activities WHERE animal_id = a.id AND activity_datetime >= CURRENT_DATE - INTERVAL '11 days' AND activity_datetime < CURRENT_DATE - INTERVAL '1 day') baseline
         ) act_dev ON true
         LEFT JOIN LATERAL (
             SELECT (r1.rumination_minutes - r2.rumination_minutes) as diff_3d
             FROM (SELECT rumination_minutes FROM ruminations WHERE animal_id = a.id ORDER BY date DESC LIMIT 1) r1,
                  LATERAL (SELECT rumination_minutes FROM ruminations WHERE animal_id = a.id ORDER BY date DESC LIMIT 1 OFFSET 2) r2
         ) rum_diff ON true
         LEFT JOIN LATERAL (
             SELECT NULL::float8 as trend, NULL::float8 as total_loss
         ) weight_trend ON true
         LEFT JOIN LATERAL (
             SELECT (mq.fat_percentage / NULLIF(mq.protein_percentage, 0))::float8 as ratio
             FROM milk_quality mq WHERE mq.animal_id = a.id AND mq.fat_percentage IS NOT NULL AND mq.protein_percentage IS NOT NULL
               AND mq.date >= CURRENT_DATE - INTERVAL '100 days' AND mq.date >= CURRENT_DATE - INTERVAL '5 days'
               AND EXISTS (SELECT 1 FROM calvings c WHERE c.animal_id = a.id AND (CURRENT_DATE - c.calving_date) BETWEEN 5 AND 100)
             ORDER BY mq.date DESC LIMIT 1
         ) fpr ON true
         LEFT JOIN LATERAL (
             SELECT CASE WHEN fd.total > 0 THEN COALESCE(fd.rest_feed::float8, 0) / fd.total * 100.0 ELSE NULL END as pct
             FROM feed_day_amounts fd WHERE fd.animal_id = a.id ORDER BY fd.feed_date DESC LIMIT 1
         ) rest_pct ON true
         LEFT JOIN LATERAL (
             SELECT milk_temperature as highest_temp FROM milk_visit_quality WHERE animal_id = a.id ORDER BY visit_datetime DESC LIMIT 1
         ) temp_highest ON true
         LEFT JOIN LATERAL (
             SELECT STRING_AGG(q || ': ' || code, ', ') as atts FROM (
                 SELECT 'LF' as q, lf_colour_code as code FROM milk_visit_quality WHERE animal_id = a.id AND lf_colour_code IS NOT NULL AND visit_datetime >= NOW() - INTERVAL '24 hours'
                 UNION ALL
                 SELECT 'LR', lr_colour_code FROM milk_visit_quality WHERE animal_id = a.id AND lr_colour_code IS NOT NULL AND visit_datetime >= NOW() - INTERVAL '24 hours'
                 UNION ALL
                 SELECT 'RF', rf_colour_code FROM milk_visit_quality WHERE animal_id = a.id AND rf_colour_code IS NOT NULL AND visit_datetime >= NOW() - INTERVAL '24 hours'
                 UNION ALL
                 SELECT 'RR', rr_colour_code FROM milk_visit_quality WHERE animal_id = a.id AND rr_colour_code IS NOT NULL AND visit_datetime >= NOW() - INTERVAL '24 hours'
             ) sub
         ) colour_atts ON true
         LEFT JOIN LATERAL (
             SELECT (short.avg - expected_curve.exp)::float8 as dev
             FROM (SELECT AVG(milk_amount)::float8 as avg FROM milk_day_productions WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '3 days') short,
                  LATERAL (SELECT AVG(milk_amount)::float8 as exp FROM milk_day_productions md JOIN calvings c ON c.animal_id = md.animal_id WHERE c.animal_id = a.id AND (md.date - c.calving_date) BETWEEN 5 AND 21 AND md.date < CURRENT_DATE - INTERVAL '3 days') expected_curve
         ) milk_trend_dev ON true
         LEFT JOIN LATERAL (
             SELECT (CURRENT_DATE - c.calving_date)::int8 as days_in_lactation FROM calvings c WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
         ) days_lac ON true
         WHERE a.active = true AND a.gender = 'female'
         ORDER BY milk_dev.dev_pct DESC NULLS LAST
         LIMIT 100"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let result: Vec<HealthTaskRow> = rows
        .into_iter()
        .filter_map(|r| {
            let milk_drop_pct_full = r.milk_deviation_pct;
            let milk_drop_kg = r.milk_drop_kg;
            let cond_highest = r.cond_highest;
            let scc = r.scc;
            let act_dev = r.act_deviation;
            let rum_diff = r.rum_diff_3d;
            let fpr = r.fat_protein_ratio;
            let rest_pct = r.feed_rest_pct;
            let colour_str = r.colour_atts.unwrap_or_default();
            let temp_highest = r.highest_temp;
            let days_in_lactation = r.days_in_lactation;
            let mut score = 0.0_f64;

            if let Some(drop) = milk_drop_pct_full {
                if drop > 20.0 {
                    score += 20.0;
                } else if drop > 10.0 {
                    score += 10.0;
                }
            }
            if let Some(cond) = cond_highest {
                if cond > 100 {
                    score += 25.0;
                } else if cond > 90 {
                    score += 15.0;
                } else if cond > 83 {
                    score += 8.0;
                }
            }
            if let Some(scc_v) = scc {
                if scc_v > 500000 {
                    score += 20.0;
                } else if scc_v > 300000 {
                    score += 12.0;
                } else if scc_v > 200000 {
                    score += 5.0;
                }
            }
            if let Some(act) = act_dev {
                if act < -30.0 {
                    score += 15.0;
                } else if act < -15.0 {
                    score += 8.0;
                }
            }
            if let Some(rum) = rum_diff {
                if rum < -120 {
                    score += 15.0;
                } else if rum < -60 {
                    score += 8.0;
                }
            }
            if let Some(fpr_v) = fpr
                && fpr_v < 1.0
            {
                score += 10.0;
            }
            if let Some(rest) = rest_pct
                && rest > 50.0
            {
                score += 8.0;
            }
            if !colour_str.is_empty() {
                score += 10.0;
            }
            if let Some(temp) = temp_highest {
                if temp > 39.5 {
                    score += 15.0;
                } else if temp > 39.0 {
                    score += 8.0;
                }
            }
            if let Some(dl) = days_in_lactation
                && dl <= 60
            {
                score *= 1.3;
            }

            if score < 10.0 {
                return None;
            }

            let status = if score >= 60.0 {
                "critical"
            } else if score >= 30.0 {
                "warning"
            } else {
                "info"
            };

            let mut cond_chronic = Vec::new();
            if let Some(cond) = cond_highest
                && cond > 80
            {
                cond_chronic.push(format!("highest={}", cond));
            }

            Some(HealthTaskRow {
                animal_id: r.animal_id,
                animal_name: r.animal_name,
                life_number: r.life_number,
                sick_chance: (score * 100.0).round() / 100.0,
                sick_chance_status: status.to_string(),
                milk_drop_kg,
                conductivity_highest: cond_highest,
                conductivity_chronic_quarters: cond_chronic,
                scc_indication: scc,
                activity_deviation: act_dev,
                rumination_deviation: rum_diff,
                weight_trend: r.weight_trend,
                total_weight_loss: r.total_weight_loss,
                fat_protein_ratio: fpr,
                feed_rest_pct: rest_pct,
                temperature_highest: temp_highest,
                colour_attentions: if colour_str.is_empty() {
                    Vec::new()
                } else {
                    colour_str.split(", ").map(String::from).collect()
                },
                milk_trend_deviation: r.milk_trend_dev,
                days_in_lactation,
            })
        })
        .collect();

    Ok(HealthTaskResponse { rows: result })
}

pub async fn pregnancy_rate_report(pool: &PgPool) -> Result<PregnancyRateResponse, AppError> {
    let today = chrono::Utc::now().date_naive();
    let mut periods = Vec::new();

    let start = today - chrono::Duration::days(9 * 7);
    let mut period_end = start + chrono::Duration::days(21);
    while period_end <= today + chrono::Duration::days(21) {
        let _period_start = period_end - chrono::Duration::days(21);
        let ins_start = period_end - chrono::Duration::days(9 * 7);
        let ins_end = period_end - chrono::Duration::days(6 * 7);

        let eligible: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT a.id)::int8 FROM animals a
             WHERE a.active = true AND a.gender = 'female'
               AND EXISTS (SELECT 1 FROM calvings c WHERE c.animal_id = a.id)
               AND EXISTS (SELECT 1 FROM inseminations i WHERE i.animal_id = a.id AND i.insemination_date >= $1 AND i.insemination_date <= $2)"
        )
        .bind(ins_start).bind(ins_end)
        .fetch_one(pool).await.map_err(AppError::Database)?;

        let inseminated: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT i.animal_id)::int8 FROM inseminations i
             JOIN animals a ON a.id = i.animal_id AND a.active = true AND a.gender = 'female'
             WHERE i.insemination_date >= $1 AND i.insemination_date <= $2",
        )
        .bind(ins_start)
        .bind(ins_end)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        let pregnant: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT p.animal_id)::int8 FROM pregnancies p
             JOIN animals a ON a.id = p.animal_id AND a.active = true AND a.gender = 'female'
             WHERE p.insemination_date >= $1 AND p.insemination_date <= $2",
        )
        .bind(ins_start)
        .bind(ins_end)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        let ins_rate = if eligible > 0 {
            Some((inseminated as f64 / eligible as f64 * 100.0 * 100.0).round() / 100.0)
        } else {
            None
        };
        let conc_rate = if inseminated > 0 {
            Some((pregnant as f64 / inseminated as f64 * 100.0 * 100.0).round() / 100.0)
        } else {
            None
        };
        let preg_rate = if eligible > 0 {
            Some((pregnant as f64 / eligible as f64 * 100.0 * 100.0).round() / 100.0)
        } else {
            None
        };

        periods.push(PregnancyRatePeriod {
            end_date: period_end.format("%Y-%m-%d").to_string(),
            eligible,
            inseminated,
            pregnant,
            insemination_rate: ins_rate,
            conception_rate: conc_rate,
            pregnancy_rate: preg_rate,
        });

        period_end += chrono::Duration::days(21);
    }

    Ok(PregnancyRateResponse { periods })
}

pub async fn transition_report(pool: &PgPool) -> Result<TransitionResponse, AppError> {
    let rows: Vec<TransitionDbRow> = sqlx::query_as(
        "SELECT a.id as animal_id, a.name as animal_name, a.life_number,
                days_rel.days as days_relative,
                milk_24h.total as milk_24h,
                NULL::float8 as sick_chance,
                rum_diff.diff_3d as rumination_3day_diff,
                rum_latest.rumination_minutes,
                feed_latest.total as feed_total,
                feed_latest.rest_feed as feed_rest,
                scc_latest.scc as latest_scc
         FROM animals a
         JOIN LATERAL (
             SELECT (CURRENT_DATE - c.calving_date)::int8 as days,
                    CASE WHEN d.dry_off_date IS NOT NULL THEN (CURRENT_DATE - d.dry_off_date)::int8 ELSE NULL END as dry_days
             FROM calvings c
             LEFT JOIN dry_offs d ON d.animal_id = a.id AND d.dry_off_date <= c.calving_date
             WHERE c.animal_id = a.id ORDER BY c.calving_date DESC LIMIT 1
         ) days_rel ON true
         LEFT JOIN LATERAL (
             SELECT SUM(milk_amount)::float8 as total FROM milk_day_productions WHERE animal_id = a.id AND date >= CURRENT_DATE - INTERVAL '1 day'
         ) milk_24h ON true
         LEFT JOIN LATERAL (
             SELECT (r1.rumination_minutes - r2.rumination_minutes) as diff_3d
             FROM (SELECT rumination_minutes FROM ruminations WHERE animal_id = a.id ORDER BY date DESC LIMIT 1) r1,
                  LATERAL (SELECT rumination_minutes FROM ruminations WHERE animal_id = a.id ORDER BY date DESC LIMIT 1 OFFSET 2) r2
         ) rum_diff ON true
         LEFT JOIN LATERAL (
             SELECT rumination_minutes FROM ruminations WHERE animal_id = a.id ORDER BY date DESC LIMIT 1
         ) rum_latest ON true
         LEFT JOIN LATERAL (
             SELECT total, rest_feed FROM feed_day_amounts WHERE animal_id = a.id ORDER BY feed_date DESC LIMIT 1
         ) feed_latest ON true
         LEFT JOIN LATERAL (
             SELECT scc FROM milk_quality WHERE animal_id = a.id ORDER BY date DESC LIMIT 1
         ) scc_latest ON true
         WHERE a.active = true AND a.gender = 'female'
           AND (days_rel.days BETWEEN -21 AND 30)
         ORDER BY days_rel.days
         LIMIT 200"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(TransitionResponse {
        rows: rows
            .into_iter()
            .map(|row| {
                TransitionRow {
                    animal_id: row.animal_id,
                    animal_name: row.animal_name,
                    life_number: row.life_number,
                    days_relative: row.days_relative,
                    milk_24h: row.milk_24h,
                    sick_chance: row.sick_chance,
                    rumination_3day_diff: row.rumination_3day_diff,
                    rumination_minutes: row.rumination_minutes,
                    feed_total: row.feed_total,
                    feed_rest: row.feed_rest,
                    latest_scc: row.latest_scc,
                }
            })
            .collect(),
    })
}

pub async fn export_milk_csv(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<String, AppError> {
    let rows: Vec<ExportMilkDbRow> = sqlx::query_as(
        "SELECT a.name as animal_name, md.date::text, md.milk_amount, md.avg_amount, md.avg_weight, md.isk
         FROM milk_day_productions md
         JOIN animals a ON a.id = md.animal_id
         WHERE ($1::date IS NULL OR md.date >= $1) AND ($2::date IS NULL OR md.date <= $2)
         ORDER BY md.date DESC, a.name",
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut csv = String::from("Животное,Дата,Надой (л),Средний надой,Средний вес,ИСК\n");
    for row in &rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            escape_csv(&row.animal_name),
            &row.date,
            row.milk_amount.map_or(String::new(), |v| format!("{:.1}", v)),
            row.avg_amount.map_or(String::new(), |v| format!("{:.1}", v)),
            row.avg_weight.map_or(String::new(), |v| format!("{:.1}", v)),
            row.isk.map_or(String::new(), |v| format!("{:.1}", v)),
        ));
    }
    Ok(csv)
}

pub async fn export_reproduction_csv(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<String, AppError> {
    let calvings: Vec<(String, String, Option<String>, Option<i32>)> = sqlx::query_as(
        "SELECT a.name, c.calving_date::text, c.remarks, c.lac_number
         FROM calvings c JOIN animals a ON a.id = c.animal_id
         WHERE ($1::date IS NULL OR c.calving_date >= $1) AND ($2::date IS NULL OR c.calving_date <= $2)
         ORDER BY c.calving_date DESC"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let inseminations: Vec<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT a.name, i.insemination_date::text, i.sire_code, i.insemination_type
         FROM inseminations i JOIN animals a ON a.id = i.animal_id
         WHERE ($1::date IS NULL OR i.insemination_date >= $1) AND ($2::date IS NULL OR i.insemination_date <= $2)
         ORDER BY i.insemination_date DESC"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut csv = String::from("=== ОТЁЛЫ ===\nЖивотное,Дата,Примечания,Лактация\n");
    for (name, date, remarks, lac) in &calvings {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            escape_csv(name),
            date,
            remarks.as_deref().map_or(String::new(), escape_csv),
            lac.map_or(String::new(), |v| v.to_string()),
        ));
    }

    csv.push_str("\n=== ИНСЕМИНАЦИИ ===\nЖивотное,Дата,Код быка,Тип\n");
    for (name, date, sire, itype) in &inseminations {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            escape_csv(name),
            date,
            sire.as_deref().map_or(String::new(), escape_csv),
            itype.as_deref().map_or(String::new(), escape_csv),
        ));
    }
    Ok(csv)
}

pub async fn export_feed_csv(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<String, AppError> {
    let rows: Vec<(String, String, i32, f64)> = sqlx::query_as(
        "SELECT a.name, f.feed_date::text, f.feed_number, f.total
         FROM feed_day_amounts f
         JOIN animals a ON a.id = f.animal_id
         WHERE ($1::date IS NULL OR f.feed_date >= $1) AND ($2::date IS NULL OR f.feed_date <= $2)
         ORDER BY f.feed_date DESC, a.name",
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    let mut csv = String::from("Животное,Дата,Номер корма,Количество (кг)\n");
    for (name, date, feed_num, total) in &rows {
        csv.push_str(&format!(
            "{},{},{},{:.1}\n",
            escape_csv(name),
            date,
            feed_num,
            total,
        ));
    }
    Ok(csv)
}

pub async fn milk_export_rows(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<Vec<Vec<String>>, AppError> {
    let rows: Vec<ExportMilkDbRow> = sqlx::query_as(
        "SELECT a.name as animal_name, md.date::text, md.milk_amount, md.avg_amount, md.avg_weight, md.isk
         FROM milk_day_productions md
         JOIN animals a ON a.id = md.animal_id
         WHERE ($1::date IS NULL OR md.date >= $1) AND ($2::date IS NULL OR md.date <= $2)
         ORDER BY md.date DESC, a.name",
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|row| {
            vec![
                row.animal_name,
                row.date,
                row.milk_amount.map_or(String::new(), |v| format!("{:.1}", v)),
                row.avg_amount.map_or(String::new(), |v| format!("{:.1}", v)),
                row.avg_weight.map_or(String::new(), |v| format!("{:.1}", v)),
                row.isk.map_or(String::new(), |v| format!("{:.1}", v)),
            ]
        })
        .collect())
}

pub async fn calvings_export_rows(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<Vec<Vec<String>>, AppError> {
    let rows: Vec<(String, String, Option<String>, Option<i32>)> = sqlx::query_as(
        "SELECT a.name, c.calving_date::text, c.remarks, c.lac_number
         FROM calvings c JOIN animals a ON a.id = c.animal_id
         WHERE ($1::date IS NULL OR c.calving_date >= $1) AND ($2::date IS NULL OR c.calving_date <= $2)
         ORDER BY c.calving_date DESC",
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|(name, date, remarks, lac)| {
            vec![
                name,
                date,
                remarks.unwrap_or_default(),
                lac.map_or(String::new(), |v| v.to_string()),
            ]
        })
        .collect())
}

pub async fn inseminations_export_rows(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<Vec<Vec<String>>, AppError> {
    let rows: Vec<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT a.name, i.insemination_date::text, i.sire_code, i.insemination_type
         FROM inseminations i JOIN animals a ON a.id = i.animal_id
         WHERE ($1::date IS NULL OR i.insemination_date >= $1) AND ($2::date IS NULL OR i.insemination_date <= $2)
         ORDER BY i.insemination_date DESC",
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|(name, date, sire, itype)| {
            vec![
                name,
                date,
                sire.unwrap_or_default(),
                itype.unwrap_or_default(),
            ]
        })
        .collect())
}

pub async fn feed_export_rows(
    pool: &PgPool,
    from_date: Option<NaiveDate>,
    till_date: Option<NaiveDate>,
) -> Result<Vec<Vec<String>>, AppError> {
    let rows: Vec<(String, String, i32, f64)> = sqlx::query_as(
        "SELECT a.name, f.feed_date::text, f.feed_number, f.total
         FROM feed_day_amounts f
         JOIN animals a ON a.id = f.animal_id
         WHERE ($1::date IS NULL OR f.feed_date >= $1) AND ($2::date IS NULL OR f.feed_date <= $2)
         ORDER BY f.feed_date DESC, a.name",
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|(name, date, feed_num, total)| {
            vec![name, date, feed_num.to_string(), format!("{:.1}", total)]
        })
        .collect())
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

trait PartialAvg {
    fn partial_avg(self) -> Option<f64>;
}

impl<I: Iterator<Item = f64>> PartialAvg for I {
    fn partial_avg(self) -> Option<f64> {
        let (sum, count) = self.fold((0.0_f64, 0_usize), |(s, c), v| (s + v, c + 1));
        if count > 0 {
            Some((sum / count as f64 * 100.0).round() / 100.0)
        } else {
            None
        }
    }
}

trait PartialSum {
    fn partial_sum(self) -> Option<f64>;
}

impl<I: Iterator<Item = f64>> PartialSum for I {
    fn partial_sum(self) -> Option<f64> {
        self.reduce(|a, b| a + b)
    }
}
