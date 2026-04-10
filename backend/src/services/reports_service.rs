use serde::Serialize;
use sqlx::PgPool;

use crate::errors::AppError;

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
    from_date: Option<chrono::NaiveDate>,
    till_date: Option<chrono::NaiveDate>,
) -> Result<MilkSummary, AppError> {
    let (total_milk,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(milk_amount), 0) FROM milk_day_productions WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (count_days,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM milk_day_productions WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (avg_per_animal,): (Option<f64>,) = sqlx::query_as(
        "SELECT AVG(milk_amount) FROM milk_day_productions WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(MilkSummary {
        total_milk,
        count_days,
        avg_per_animal,
    })
}

pub async fn reproduction_summary(
    pool: &PgPool,
    from_date: Option<chrono::NaiveDate>,
    till_date: Option<chrono::NaiveDate>,
) -> Result<ReproductionSummary, AppError> {
    let (total_calvings,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM calvings WHERE ($1::date IS NULL OR calving_date >= $1) AND ($2::date IS NULL OR calving_date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(|e| { tracing::error!("calvings query error: {:?}", e); AppError::Database(e) })?;

    let (total_inseminations,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM inseminations WHERE ($1::date IS NULL OR insemination_date >= $1) AND ($2::date IS NULL OR insemination_date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (total_pregnancies,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pregnancies WHERE ($1::date IS NULL OR pregnancy_date >= $1) AND ($2::date IS NULL OR pregnancy_date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (total_heats,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM heats WHERE ($1::date IS NULL OR heat_date >= $1) AND ($2::date IS NULL OR heat_date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (total_dry_offs,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM dry_offs WHERE ($1::date IS NULL OR dry_off_date >= $1) AND ($2::date IS NULL OR dry_off_date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

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
    from_date: Option<chrono::NaiveDate>,
    till_date: Option<chrono::NaiveDate>,
) -> Result<FeedSummary, AppError> {
    let (total_feed,): (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total), 0) FROM feed_day_amounts WHERE ($1::date IS NULL OR feed_date >= $1) AND ($2::date IS NULL OR feed_date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    let (total_visits,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM feed_visits WHERE ($1::date IS NULL OR visit_datetime::date >= $1) AND ($2::date IS NULL OR visit_datetime::date <= $2)"
    )
    .bind(from_date)
    .bind(till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(FeedSummary {
        total_feed_kg: total_feed,
        total_visits,
    })
}

pub async fn export_milk_csv(
    pool: &PgPool,
    from_date: Option<chrono::NaiveDate>,
    till_date: Option<chrono::NaiveDate>,
) -> Result<String, AppError> {
    let rows: Vec<(
        String,
        String,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<f64>,
    )> = sqlx::query_as(
        "SELECT a.name, md.date::text, md.milk_amount, md.avg_amount, md.avg_weight, md.isk
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
    for (name, date, milk, avg_amount, avg_weight, isk) in &rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            escape_csv(name),
            date,
            milk.map_or(String::new(), |v| format!("{:.1}", v)),
            avg_amount.map_or(String::new(), |v| format!("{:.1}", v)),
            avg_weight.map_or(String::new(), |v| format!("{:.1}", v)),
            isk.map_or(String::new(), |v| format!("{:.1}", v)),
        ));
    }
    Ok(csv)
}

pub async fn export_reproduction_csv(
    pool: &PgPool,
    from_date: Option<chrono::NaiveDate>,
    till_date: Option<chrono::NaiveDate>,
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
    from_date: Option<chrono::NaiveDate>,
    till_date: Option<chrono::NaiveDate>,
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

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
