use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::pagination::paginated;
use crate::models::sire::{CreateSire, SireFilter, UpdateSire};
use crate::services::sire_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/sires", get(list).post(create))
		.route("/sires/{id}", get(get_by_id).put(update).delete(remove))
}

#[utoipa::path(
	get,
	path = "/api/v1/sires",
	responses(
		(status = 200, description = "List of sires", body = serde_json::Value),
		(status = 401, description = "Unauthorized")
	),
	params(SireFilter),
	security(("cookie_auth" = []))
)]
async fn list(
	_claims: Claims,
	State(state): State<AppState>,
	Query(filter): Query<SireFilter>,
) -> Result<Json<Value>, AppError> {
	let pool = &state.pool;
	let f = &filter;
	paginated(
		filter.page,
		filter.per_page,
		|| sire_service::list(pool, f),
		|| sire_service::count(pool, f),
	)
	.await
}

#[utoipa::path(
	get,
	path = "/api/v1/sires/{id}",
	responses(
		(status = 200, description = "Sire found", body = serde_json::Value),
		(status = 404, description = "Not found"),
		(status = 401, description = "Unauthorized")
	),
	params(("id" = i32, Path, description = "Sire ID")),
	security(("cookie_auth" = []))
)]
async fn get_by_id(
	_claims: Claims,
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
	let item = sire_service::get_by_id(&state.pool, id)
		.await?
		.ok_or_else(|| AppError::NotFound(format!("Бык {} не найден", id)))?;
	Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
	post,
	path = "/api/v1/sires",
	request_body = CreateSire,
	responses(
		(status = 201, description = "Sire created", body = serde_json::Value),
		(status = 400, description = "Validation error"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Admin access required")
	),
	security(("cookie_auth" = []))
)]
async fn create(
	_admin: crate::middleware::auth::AdminGuard,
	State(state): State<AppState>,
	Json(req): Json<CreateSire>,
) -> Result<Json<Value>, AppError> {
	req.validate()?;
	let item = sire_service::create(&state.pool, &req).await?;
	Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
	put,
	path = "/api/v1/sires/{id}",
	request_body = UpdateSire,
	responses(
		(status = 200, description = "Sire updated", body = serde_json::Value),
		(status = 404, description = "Not found"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Admin access required")
	),
	params(("id" = i32, Path, description = "Sire ID")),
	security(("cookie_auth" = []))
)]
async fn update(
	_admin: crate::middleware::auth::AdminGuard,
	State(state): State<AppState>,
	Path(id): Path<i32>,
	Json(req): Json<UpdateSire>,
) -> Result<Json<Value>, AppError> {
	req.validate()?;
	let item = sire_service::update(&state.pool, id, &req).await?;
	Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
	delete,
	path = "/api/v1/sires/{id}",
	responses(
		(status = 200, description = "Sire deleted", body = serde_json::Value),
		(status = 404, description = "Not found"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Admin access required")
	),
	params(("id" = i32, Path, description = "Sire ID")),
	security(("cookie_auth" = []))
)]
async fn remove(
	_admin: crate::middleware::auth::AdminGuard,
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
	sire_service::delete(&state.pool, id).await?;
	Ok(Json(json!({ "message": "Удалено" })))
}
