use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::vet::{
    CreateVetRecord, CreateWeightRecord, VetRecord, VetRecordFilter, VetRecordStatus,
    WeightRecord, WeightRecordFilter,
};

pub async fn list_vet_records(
    pool: &PgPool,
    filter: &VetRecordFilter,
) -> Result<Vec<VetRecord>, AppError> {
    let pag = crate::models::pagination::Pagination::new(filter.page, filter.per_page, 20, 100);
    sqlx::query_as::<_, VetRecord>(
        "SELECT * FROM vet_records
         WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::vet_record_type IS NULL OR record_type = $2)
         AND ($3::vet_record_status IS NULL OR status = $3)
         AND ($4::date IS NULL OR event_date >= $4)
         AND ($5::date IS NULL OR event_date <= $5)
         ORDER BY event_date DESC, id DESC
         LIMIT $6 OFFSET $7",
    )
    .bind(filter.animal_id)
    .bind(&filter.record_type)
    .bind(&filter.status)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_vet_records(
    pool: &PgPool,
    filter: &VetRecordFilter,
) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM vet_records
         WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::vet_record_type IS NULL OR record_type = $2)
         AND ($3::vet_record_status IS NULL OR status = $3)
         AND ($4::date IS NULL OR event_date >= $4)
         AND ($5::date IS NULL OR event_date <= $5)",
    )
    .bind(filter.animal_id)
    .bind(&filter.record_type)
    .bind(&filter.status)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn get_vet_record(pool: &PgPool, id: i32) -> Result<Option<VetRecord>, AppError> {
    sqlx::query_as::<_, VetRecord>("SELECT * FROM vet_records WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create_vet_record(
    pool: &PgPool,
    req: &CreateVetRecord,
) -> Result<VetRecord, AppError> {
    crate::services::animal_service::ensure_exists(pool, req.animal_id).await?;

    let withdrawal_end_date = match (req.withdrawal_days, req.event_date) {
        (Some(days), date) if days > 0 => Some(
            date + chrono::Duration::try_days(days as i64).unwrap(),
        ),
        _ => None,
    };

    let status = req
        .status
        .as_ref()
        .cloned()
        .unwrap_or(VetRecordStatus::Completed);

    sqlx::query_as::<_, VetRecord>(
        "INSERT INTO vet_records (animal_id, record_type, status, event_date,
         diagnosis, treatment, medication, dosage, withdrawal_days, withdrawal_end_date,
         veterinarian, notes, follow_up_date)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) RETURNING *",
    )
    .bind(req.animal_id)
    .bind(&req.record_type)
    .bind(&status)
    .bind(req.event_date)
    .bind(&req.diagnosis)
    .bind(&req.treatment)
    .bind(&req.medication)
    .bind(&req.dosage)
    .bind(req.withdrawal_days)
    .bind(withdrawal_end_date)
    .bind(&req.veterinarian)
    .bind(&req.notes)
    .bind(req.follow_up_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_vet_record(
    pool: &PgPool,
    id: i32,
    req: &crate::models::vet::UpdateVetRecord,
) -> Result<VetRecord, AppError> {
    let existing = get_vet_record(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Ветеринарная запись с ID {} не найдена", id)))?;

    let withdrawal_days = req.withdrawal_days.or(existing.withdrawal_days);
    let event_date = req.event_date.unwrap_or(existing.event_date);
    let withdrawal_end_date = match withdrawal_days {
        Some(days) if days > 0 => Some(
            event_date + chrono::Duration::try_days(days as i64).unwrap(),
        ),
        _ => None,
    };

    sqlx::query_as::<_, VetRecord>(
        "UPDATE vet_records SET
         record_type = COALESCE($2, record_type),
         status = COALESCE($3, status),
         event_date = COALESCE($4, event_date),
         diagnosis = COALESCE($5, diagnosis),
         treatment = COALESCE($6, treatment),
         medication = COALESCE($7, medication),
         dosage = COALESCE($8, dosage),
         withdrawal_days = COALESCE($9, withdrawal_days),
         withdrawal_end_date = $10,
         veterinarian = COALESCE($11, veterinarian),
         notes = COALESCE($12, notes),
         follow_up_date = COALESCE($13, follow_up_date),
         updated_at = NOW()
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(&req.record_type)
    .bind(&req.status)
    .bind(req.event_date)
    .bind(&req.diagnosis)
    .bind(&req.treatment)
    .bind(&req.medication)
    .bind(&req.dosage)
    .bind(req.withdrawal_days)
    .bind(withdrawal_end_date)
    .bind(&req.veterinarian)
    .bind(&req.notes)
    .bind(req.follow_up_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_vet_record(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM vet_records WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Ветеринарная запись с ID {} не найдена",
            id
        )));
    }
    Ok(())
}

pub async fn list_weight_records(
    pool: &PgPool,
    filter: &WeightRecordFilter,
) -> Result<Vec<WeightRecord>, AppError> {
    let pag = crate::models::pagination::Pagination::new(filter.page, filter.per_page, 20, 100);
    sqlx::query_as::<_, WeightRecord>(
        "SELECT * FROM weight_records
         WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::date IS NULL OR measure_date >= $2)
         AND ($3::date IS NULL OR measure_date <= $3)
         ORDER BY measure_date DESC, id DESC
         LIMIT $4 OFFSET $5",
    )
    .bind(filter.animal_id)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_weight_records(
    pool: &PgPool,
    filter: &WeightRecordFilter,
) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM weight_records
         WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::date IS NULL OR measure_date >= $2)
         AND ($3::date IS NULL OR measure_date <= $3)",
    )
    .bind(filter.animal_id)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create_weight_record(
    pool: &PgPool,
    req: &CreateWeightRecord,
) -> Result<WeightRecord, AppError> {
    crate::services::animal_service::ensure_exists(pool, req.animal_id).await?;

    sqlx::query_as::<_, WeightRecord>(
        "INSERT INTO weight_records (animal_id, weight_kg, bcs, measure_date, notes)
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(req.animal_id)
    .bind(req.weight_kg)
    .bind(req.bcs)
    .bind(req.measure_date)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_weight_record(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM weight_records WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Запись веса с ID {} не найдена",
            id
        )));
    }
    Ok(())
}

pub async fn upcoming_follow_ups(pool: &PgPool, days: i32) -> Result<Vec<VetRecord>, AppError> {
    sqlx::query_as::<_, VetRecord>(
        "SELECT v.* FROM vet_records v
         JOIN animals a ON a.id = v.animal_id AND a.active = true
         WHERE v.follow_up_date IS NOT NULL
         AND v.follow_up_date BETWEEN CURRENT_DATE AND CURRENT_DATE + make_interval(days => $1)
         AND v.status NOT IN ('completed', 'cancelled')
         ORDER BY v.follow_up_date",
    )
    .bind(days)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn active_withdrawals(pool: &PgPool) -> Result<Vec<VetRecord>, AppError> {
    sqlx::query_as::<_, VetRecord>(
        "SELECT v.* FROM vet_records v
         JOIN animals a ON a.id = v.animal_id AND a.active = true
         WHERE v.withdrawal_end_date IS NOT NULL
         AND v.withdrawal_end_date >= CURRENT_DATE
         AND v.status = 'completed'
         ORDER BY v.withdrawal_end_date",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}
