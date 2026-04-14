use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::transfer::{CreateTransfer, Transfer, TransferFilter, UpdateTransfer};

pub async fn list(pool: &PgPool, filter: &TransferFilter) -> Result<Vec<Transfer>, AppError> {
	let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
	sqlx::query_as::<_, Transfer>(
		"SELECT * FROM transfers WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
		 AND ($2::text IS NULL OR transfer_type = $2)
		 ORDER BY transfer_date DESC LIMIT $3 OFFSET $4",
	)
	.bind(filter.animal_id.clone())
	.bind(filter.transfer_type.clone())
	.bind(pag.per_page)
	.bind(pag.offset)
	.fetch_all(pool)
	.await
	.map_err(AppError::Database)
}

pub async fn count(pool: &PgPool, filter: &TransferFilter) -> Result<i64, AppError> {
	let row: (i64,) = sqlx::query_as(
		"SELECT COUNT(*) FROM transfers WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
		 AND ($2::text IS NULL OR transfer_type = $2)",
	)
	.bind(filter.animal_id.clone())
	.bind(filter.transfer_type.clone())
	.fetch_one(pool)
	.await
	.map_err(AppError::Database)?;
	Ok(row.0)
}

pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Transfer>, AppError> {
	sqlx::query_as::<_, Transfer>("SELECT * FROM transfers WHERE id = $1")
		.bind(id)
		.fetch_optional(pool)
		.await
		.map_err(AppError::Database)
}

pub async fn create(pool: &PgPool, req: &CreateTransfer) -> Result<Transfer, AppError> {
	let mut tx = pool.begin().await.map_err(AppError::Database)?;

	let exists: bool =
		sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM animals WHERE id = $1 AND active = true)")
			.bind(req.animal_id)
			.fetch_one(&mut *tx)
			.await
			.map_err(AppError::Database)?;
	if !exists {
		return Err(AppError::NotFound(format!(
			"Животное с ID {} не найдено или неактивно",
			req.animal_id
		)));
	}

	let row = sqlx::query_as::<_, Transfer>(
		"INSERT INTO transfers (animal_id, transfer_date, transfer_type, reason_id, from_location, to_location)
		 VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
	)
	.bind(req.animal_id)
	.bind(req.transfer_date)
	.bind(&req.transfer_type)
	.bind(req.reason_id)
	.bind(&req.from_location)
	.bind(&req.to_location)
	.fetch_one(&mut *tx)
	.await
	.map_err(AppError::Database)?;

	tx.commit().await.map_err(AppError::Database)?;
	Ok(row)
}

pub async fn update(pool: &PgPool, id: i32, req: &UpdateTransfer) -> Result<Transfer, AppError> {
	sqlx::query_as::<_, Transfer>(
		"UPDATE transfers SET
		 transfer_date = COALESCE($2, transfer_date),
		 transfer_type = COALESCE($3, transfer_type),
		 reason_id = COALESCE($4, reason_id),
		 from_location = COALESCE($5, from_location),
		 to_location = COALESCE($6, to_location)
		 WHERE id = $1 RETURNING *",
	)
	.bind(id)
	.bind(req.transfer_date)
	.bind(&req.transfer_type)
	.bind(req.reason_id)
	.bind(&req.from_location)
	.bind(&req.to_location)
	.fetch_one(pool)
	.await
	.map_err(AppError::Database)
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<(), AppError> {
	let result = sqlx::query("DELETE FROM transfers WHERE id = $1")
		.bind(id)
		.execute(pool)
		.await
		.map_err(AppError::Database)?;
	if result.rows_affected() == 0 {
		return Err(AppError::NotFound(format!(
			"Перемещение {} не найдено",
			id
		)));
	}
	Ok(())
}
