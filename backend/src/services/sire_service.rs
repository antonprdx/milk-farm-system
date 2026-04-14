use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::sire::{CreateSire, Sire, SireFilter, UpdateSire};

pub async fn list(pool: &PgPool, filter: &SireFilter) -> Result<Vec<Sire>, AppError> {
	let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
	sqlx::query_as::<_, Sire>(
		"SELECT * FROM sires WHERE ($1::text IS NULL OR sire_code ILIKE '%' || $1 || '%'
		 OR life_number ILIKE '%' || $1 || '%' OR name ILIKE '%' || $1 || '%')
		 ORDER BY name LIMIT $2 OFFSET $3",
	)
	.bind(filter.search.clone())
	.bind(pag.per_page)
	.bind(pag.offset)
	.fetch_all(pool)
	.await
	.map_err(AppError::Database)
}

pub async fn count(pool: &PgPool, filter: &SireFilter) -> Result<i64, AppError> {
	let row: (i64,) = sqlx::query_as(
		"SELECT COUNT(*) FROM sires WHERE ($1::text IS NULL OR sire_code ILIKE '%' || $1 || '%'
		 OR life_number ILIKE '%' || $1 || '%' OR name ILIKE '%' || $1 || '%')",
	)
	.bind(filter.search.clone())
	.fetch_one(pool)
	.await
	.map_err(AppError::Database)?;
	Ok(row.0)
}

pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Sire>, AppError> {
	sqlx::query_as::<_, Sire>("SELECT * FROM sires WHERE id = $1")
		.bind(id)
		.fetch_optional(pool)
		.await
		.map_err(AppError::Database)
}

pub async fn create(pool: &PgPool, req: &CreateSire) -> Result<Sire, AppError> {
	sqlx::query_as::<_, Sire>(
		"INSERT INTO sires (sire_code, life_number, name, active)
		 VALUES ($1, $2, $3, $4) RETURNING *",
	)
	.bind(&req.sire_code)
	.bind(&req.life_number)
	.bind(&req.name)
	.bind(req.active)
	.fetch_one(pool)
	.await
	.map_err(AppError::Database)
}

pub async fn update(pool: &PgPool, id: i32, req: &UpdateSire) -> Result<Sire, AppError> {
	let active_set = req.active.is_some();
	let active_val = req.active;
	sqlx::query_as::<_, Sire>(
		"UPDATE sires SET
		 sire_code = COALESCE($2, sire_code),
		 life_number = COALESCE($3, life_number),
		 name = COALESCE($4, name),
		 active = CASE WHEN $5 THEN $6 ELSE active END
		 WHERE id = $1 RETURNING *",
	)
	.bind(id)
	.bind(&req.sire_code)
	.bind(&req.life_number)
	.bind(&req.name)
	.bind(active_set)
	.bind(active_val)
	.fetch_one(pool)
	.await
	.map_err(AppError::Database)
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<(), AppError> {
	let result = sqlx::query("DELETE FROM sires WHERE id = $1")
		.bind(id)
		.execute(pool)
		.await
		.map_err(AppError::Database)?;
	if result.rows_affected() == 0 {
		return Err(AppError::NotFound(format!("Бык {} не найден", id)));
	}
	Ok(())
}
