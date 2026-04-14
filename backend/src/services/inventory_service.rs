use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::inventory::*;

fn escape_like(s: &str) -> String {
	s.replace('\\', "\\\\")
		.replace('%', "\\%")
		.replace('_', "\\_")
}

pub async fn list_items(
	pool: &PgPool,
	filter: &InventoryFilter,
) -> Result<Vec<InventoryItem>, AppError> {
	let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
	let search_pattern = filter
		.search
		.as_ref()
		.map(|s| format!("%{}%", escape_like(s)));

	sqlx::query_as::<_, InventoryItem>(
		"SELECT * FROM inventory_items
		 WHERE ($1::text IS NULL OR category = $1)
		 AND ($2::bool IS NULL OR ($2 AND quantity <= min_quantity) OR (NOT $2 AND quantity > min_quantity))
		 AND ($3::text IS NULL OR name ILIKE $3)
		 ORDER BY name LIMIT $4 OFFSET $5",
	)
	.bind(&filter.category)
	.bind(filter.low_stock)
	.bind(&search_pattern)
	.bind(pag.per_page)
	.bind(pag.offset)
	.fetch_all(pool)
	.await
	.map_err(AppError::Database)
}

pub async fn count_items(pool: &PgPool, filter: &InventoryFilter) -> Result<i64, AppError> {
	let search_pattern = filter
		.search
		.as_ref()
		.map(|s| format!("%{}%", escape_like(s)));

	let row: (i64,) = sqlx::query_as(
		"SELECT COUNT(*) FROM inventory_items
		 WHERE ($1::text IS NULL OR category = $1)
		 AND ($2::bool IS NULL OR ($2 AND quantity <= min_quantity) OR (NOT $2 AND quantity > min_quantity))
		 AND ($3::text IS NULL OR name ILIKE $3)",
	)
	.bind(&filter.category)
	.bind(filter.low_stock)
	.bind(&search_pattern)
	.fetch_one(pool)
	.await
	.map_err(AppError::Database)?;
	Ok(row.0)
}

pub async fn get_item(pool: &PgPool, id: i32) -> Result<Option<InventoryItem>, AppError> {
	sqlx::query_as::<_, InventoryItem>("SELECT * FROM inventory_items WHERE id = $1")
		.bind(id)
		.fetch_optional(pool)
		.await
		.map_err(AppError::Database)
}

pub async fn create_item(pool: &PgPool, data: &CreateInventoryItem) -> Result<InventoryItem, AppError> {
	sqlx::query_as::<_, InventoryItem>(
		"INSERT INTO inventory_items (name, category, unit, quantity, min_quantity, cost_per_unit, supplier, notes)
		 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
	)
	.bind(&data.name)
	.bind(&data.category)
	.bind(data.unit.as_deref().unwrap_or("pcs"))
	.bind(data.quantity.unwrap_or(0.0))
	.bind(data.min_quantity.unwrap_or(0.0))
	.bind(data.cost_per_unit)
	.bind(&data.supplier)
	.bind(&data.notes)
	.fetch_one(pool)
	.await
	.map_err(AppError::Database)
}

pub async fn update_item(
	pool: &PgPool,
	id: i32,
	data: &UpdateInventoryItem,
) -> Result<InventoryItem, AppError> {
	let existing = get_item(pool, id)
		.await?
		.ok_or_else(|| AppError::NotFound("Позиция склада не найдена".into()))?;

	sqlx::query_as::<_, InventoryItem>(
		"UPDATE inventory_items SET name = $1, category = $2, unit = $3,
		 min_quantity = $4, cost_per_unit = $5, supplier = $6, notes = $7,
		 updated_at = NOW() WHERE id = $8 RETURNING *",
	)
	.bind(data.name.as_deref().unwrap_or(&existing.name))
	.bind(data.category.as_deref().unwrap_or(&existing.category))
	.bind(data.unit.as_deref().or(Some(&existing.unit)).unwrap_or("pcs"))
	.bind(data.min_quantity.unwrap_or(existing.min_quantity))
	.bind(data.cost_per_unit.or(existing.cost_per_unit))
	.bind(data.supplier.as_deref().or(existing.supplier.as_deref()))
	.bind(data.notes.as_deref().or(existing.notes.as_deref()))
	.bind(id)
	.fetch_one(pool)
	.await
	.map_err(AppError::Database)
}

pub async fn delete_item(pool: &PgPool, id: i32) -> Result<(), AppError> {
	let result = sqlx::query("DELETE FROM inventory_items WHERE id = $1")
		.bind(id)
		.execute(pool)
		.await
		.map_err(AppError::Database)?;
	if result.rows_affected() == 0 {
		return Err(AppError::NotFound("Позиция склада не найдена".into()));
	}
	Ok(())
}

pub async fn list_transactions(
	pool: &PgPool,
	item_id: i32,
	page: Option<i64>,
	per_page: Option<i64>,
) -> Result<Vec<InventoryTransaction>, AppError> {
	let pag = crate::models::pagination::Pagination::from_filter(page, per_page);

	sqlx::query_as::<_, InventoryTransaction>(
		"SELECT * FROM inventory_transactions WHERE item_id = $1
		 ORDER BY transaction_date DESC, id DESC LIMIT $2 OFFSET $3",
	)
	.bind(item_id)
	.bind(pag.per_page)
	.bind(pag.offset)
	.fetch_all(pool)
	.await
	.map_err(AppError::Database)
}

pub async fn create_transaction(
	pool: &PgPool,
	data: &CreateInventoryTransaction,
) -> Result<InventoryTransaction, AppError> {
	let delta = match data.transaction_type.as_str() {
		"in" => data.quantity,
		"out" => -data.quantity,
		"adjustment" => data.quantity,
		_ => return Err(AppError::BadRequest("Неверный тип транзакции".into())),
	};

	let mut tx = pool.begin().await.map_err(AppError::Database)?;

	let updated = sqlx::query(
		"UPDATE inventory_items SET quantity = quantity + $1, updated_at = NOW() WHERE id = $2",
	)
	.bind(delta)
	.bind(data.item_id)
	.execute(&mut *tx)
	.await
	.map_err(AppError::Database)?;

	if updated.rows_affected() == 0 {
		return Err(AppError::NotFound("Позиция склада не найдена".into()));
	}

	let transaction = sqlx::query_as::<_, InventoryTransaction>(
		"INSERT INTO inventory_transactions (item_id, transaction_type, quantity, notes, transaction_date)
		 VALUES ($1, $2, $3, $4, COALESCE($5, CURRENT_DATE)) RETURNING *",
	)
	.bind(data.item_id)
	.bind(&data.transaction_type)
	.bind(data.quantity)
	.bind(&data.notes)
	.bind(data.transaction_date)
	.fetch_one(&mut *tx)
	.await
	.map_err(AppError::Database)?;

	tx.commit().await.map_err(AppError::Database)?;
	Ok(transaction)
}

pub async fn low_stock_items(pool: &PgPool) -> Result<Vec<InventoryItem>, AppError> {
	sqlx::query_as::<_, InventoryItem>(
		"SELECT * FROM inventory_items WHERE quantity <= min_quantity ORDER BY name",
	)
	.fetch_all(pool)
	.await
	.map_err(AppError::Database)
}

#[cfg(test)]
mod tests {
	use super::*;

	async fn seed_item(pool: &PgPool) -> i32 {
		let row: (i32,) = sqlx::query_as(
			"INSERT INTO inventory_items (name, category, unit, quantity, min_quantity) VALUES ('Test Feed', 'feed', 'kg', 100.0, 10.0) RETURNING id",
		)
		.fetch_one(pool)
		.await
		.unwrap();
		row.0
	}

	#[sqlx::test(migrations = "./migrations")]
	async fn test_list_items_empty(pool: PgPool) {
		let filter = InventoryFilter {
			category: None,
			low_stock: None,
			search: None,
			page: None,
			per_page: None,
		};
		let items = list_items(&pool, &filter).await.unwrap();
		assert!(items.is_empty());
	}

	#[sqlx::test(migrations = "./migrations")]
	async fn test_create_item(pool: PgPool) {
		let data = CreateInventoryItem {
			name: "Сено".into(),
			category: "feed".into(),
			unit: Some("kg".into()),
			quantity: Some(500.0),
			min_quantity: Some(50.0),
			cost_per_unit: Some(5.0),
			supplier: None,
			notes: None,
		};
		let item = create_item(&pool, &data).await.unwrap();
		assert_eq!(item.name, "Сено");
		assert_eq!(item.category, "feed");
	}

	#[sqlx::test(migrations = "./migrations")]
	async fn test_update_item(pool: PgPool) {
		let id = seed_item(&pool).await;
		let data = UpdateInventoryItem {
			name: Some("Updated Feed".into()),
			category: None,
			unit: None,
			min_quantity: Some(20.0),
			cost_per_unit: None,
			supplier: None,
			notes: None,
		};
		let updated = update_item(&pool, id, &data).await.unwrap();
		assert_eq!(updated.name, "Updated Feed");
		assert_eq!(updated.min_quantity, 20.0);
	}

	#[sqlx::test(migrations = "./migrations")]
	async fn test_delete_item(pool: PgPool) {
		let id = seed_item(&pool).await;
		delete_item(&pool, id).await.unwrap();
		let found = get_item(&pool, id).await.unwrap();
		assert!(found.is_none());
	}

	#[sqlx::test(migrations = "./migrations")]
	async fn test_create_transaction_in(pool: PgPool) {
		let id = seed_item(&pool).await;
		let data = CreateInventoryTransaction {
			item_id: id,
			transaction_type: "in".into(),
			quantity: 50.0,
			notes: None,
			transaction_date: None,
		};
		let tx = create_transaction(&pool, &data).await.unwrap();
		assert_eq!(tx.transaction_type, "in");
		let item = get_item(&pool, id).await.unwrap().unwrap();
		assert_eq!(item.quantity, 150.0);
	}

	#[sqlx::test(migrations = "./migrations")]
	async fn test_create_transaction_out(pool: PgPool) {
		let id = seed_item(&pool).await;
		let data = CreateInventoryTransaction {
			item_id: id,
			transaction_type: "out".into(),
			quantity: 30.0,
			notes: None,
			transaction_date: None,
		};
		create_transaction(&pool, &data).await.unwrap();
		let item = get_item(&pool, id).await.unwrap().unwrap();
		assert_eq!(item.quantity, 70.0);
	}

	#[sqlx::test(migrations = "./migrations")]
	async fn test_low_stock_items(pool: PgPool) {
		let row: (i32,) = sqlx::query_as(
			"INSERT INTO inventory_items (name, category, unit, quantity, min_quantity) VALUES ('Low Item', 'feed', 'kg', 5.0, 10.0) RETURNING id",
		)
		.fetch_one(&pool)
		.await
		.unwrap();
		let items = low_stock_items(&pool).await.unwrap();
		assert_eq!(items.len(), 1);
		assert_eq!(items[0].id, row.0);
	}

	#[sqlx::test(migrations = "./migrations")]
	async fn test_filter_by_category(pool: PgPool) {
		seed_item(&pool).await;
		sqlx::query(
			"INSERT INTO inventory_items (name, category, unit, quantity, min_quantity) VALUES ('Med', 'medicine', 'pcs', 10.0, 2.0)",
		)
		.execute(&pool)
		.await
		.unwrap();
		let filter = InventoryFilter {
			category: Some("medicine".into()),
			low_stock: None,
			search: None,
			page: None,
			per_page: None,
		};
		let items = list_items(&pool, &filter).await.unwrap();
		assert_eq!(items.len(), 1);
		assert_eq!(items[0].category, "medicine");
	}
}
