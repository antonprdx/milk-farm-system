use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, Claims};
use crate::models::inventory::{
	CreateInventoryItem, CreateInventoryTransaction, InventoryFilter, UpdateInventoryItem,
};
use crate::models::pagination::paginated;
use crate::services::inventory_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/inventory", get(list_items).post(create_item))
		.route("/inventory/low-stock", get(low_stock))
		.route(
			"/inventory/{id}",
			get(get_item).put(update_item).delete(delete_item),
		)
		.route("/inventory/{id}/transaction", post(create_transaction))
}

#[utoipa::path(
	get,
	path = "/api/v1/inventory",
	responses(
		(status = 200, description = "List of inventory items", body = serde_json::Value),
		(status = 401, description = "Unauthorized")
	),
	params(InventoryFilter),
	security(("cookie_auth" = []))
)]
async fn list_items(
	_claims: Claims,
	State(state): State<AppState>,
	Query(filter): Query<InventoryFilter>,
) -> Result<Json<Value>, AppError> {
	let pool = &state.pool;
	let f = &filter;
	paginated(
		filter.page,
		filter.per_page,
		|| inventory_service::list_items(pool, f),
		|| inventory_service::count_items(pool, f),
	)
	.await
}

#[utoipa::path(
	get,
	path = "/api/v1/inventory/low-stock",
	responses(
		(status = 200, description = "Low stock items", body = serde_json::Value),
		(status = 401, description = "Unauthorized")
	),
	security(("cookie_auth" = []))
)]
async fn low_stock(
	_claims: Claims,
	State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
	let items = inventory_service::low_stock_items(&state.pool).await?;
	Ok(Json(json!({ "data": items, "total": items.len() })))
}

#[utoipa::path(
	get,
	path = "/api/v1/inventory/{id}",
	responses(
		(status = 200, description = "Inventory item found", body = serde_json::Value),
		(status = 404, description = "Not found"),
		(status = 401, description = "Unauthorized")
	),
	params(("id" = i32, Path, description = "Item ID")),
	security(("cookie_auth" = []))
)]
async fn get_item(
	_claims: Claims,
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
	let item = inventory_service::get_item(&state.pool, id)
		.await?
		.ok_or_else(|| AppError::NotFound("Позиция склада не найдена".into()))?;
	Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
	post,
	path = "/api/v1/inventory",
	request_body = CreateInventoryItem,
	responses(
		(status = 200, description = "Item created", body = serde_json::Value),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Admin access required")
	),
	security(("cookie_auth" = []))
)]
async fn create_item(
	_admin: AdminGuard,
	State(state): State<AppState>,
	Json(data): Json<CreateInventoryItem>,
) -> Result<Json<Value>, AppError> {
	data.validate()?;
	let item = inventory_service::create_item(&state.pool, &data).await?;
	Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
	put,
	path = "/api/v1/inventory/{id}",
	request_body = UpdateInventoryItem,
	responses(
		(status = 200, description = "Item updated", body = serde_json::Value),
		(status = 404, description = "Not found"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Admin access required")
	),
	params(("id" = i32, Path, description = "Item ID")),
	security(("cookie_auth" = []))
)]
async fn update_item(
	_admin: AdminGuard,
	State(state): State<AppState>,
	Path(id): Path<i32>,
	Json(data): Json<UpdateInventoryItem>,
) -> Result<Json<Value>, AppError> {
	data.validate()?;
	let item = inventory_service::update_item(&state.pool, id, &data).await?;
	Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
	delete,
	path = "/api/v1/inventory/{id}",
	responses(
		(status = 200, description = "Item deleted"),
		(status = 404, description = "Not found"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Admin access required")
	),
	params(("id" = i32, Path, description = "Item ID")),
	security(("cookie_auth" = []))
)]
async fn delete_item(
	_admin: AdminGuard,
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
	inventory_service::delete_item(&state.pool, id).await?;
	Ok(Json(json!({ "message": "Позиция удалена" })))
}

#[utoipa::path(
	post,
	path = "/api/v1/inventory/{id}/transaction",
	request_body = CreateInventoryTransaction,
	responses(
		(status = 200, description = "Transaction created", body = serde_json::Value),
		(status = 404, description = "Item not found"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Admin access required")
	),
	params(("id" = i32, Path, description = "Item ID")),
	security(("cookie_auth" = []))
)]
async fn create_transaction(
	_admin: AdminGuard,
	State(state): State<AppState>,
	Path(id): Path<i32>,
	Json(mut data): Json<CreateInventoryTransaction>,
) -> Result<Json<Value>, AppError> {
	data.item_id = id;
	data.validate()?;
	let tx = inventory_service::create_transaction(&state.pool, &data).await?;
	Ok(Json(json!({ "data": tx })))
}
