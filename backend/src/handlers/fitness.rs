use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::fitness::{CreateActivity, CreateRumination, FitnessFilter};
use crate::models::pagination::paginated;
use crate::services::fitness_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/fitness/activities", get(list_activities).post(create_activity))
		.route(
			"/fitness/ruminations",
			get(list_ruminations).post(create_rumination),
		)
}

#[utoipa::path(
    get,
    path = "/api/v1/fitness/activities",
    responses(
        (status = 200, description = "List of activities", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(FitnessFilter),
    security(("cookie_auth" = []))
)]
async fn list_activities(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FitnessFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || fitness_service::list_activities(pool, f),
        || fitness_service::count_activities(pool, f),
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/fitness/ruminations",
    responses(
        (status = 200, description = "List of ruminations", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(FitnessFilter),
    security(("cookie_auth" = []))
)]
async fn list_ruminations(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<FitnessFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
	paginated(
		filter.page,
		filter.per_page,
		|| fitness_service::list_ruminations(pool, f),
		|| fitness_service::count_ruminations(pool, f),
	)
	.await
}

#[utoipa::path(
	post,
	path = "/api/v1/fitness/activities",
	request_body = CreateActivity,
	responses(
		(status = 201, description = "Activity created", body = serde_json::Value),
		(status = 400, description = "Validation error"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Admin access required")
	),
	security(("cookie_auth" = []))
)]
async fn create_activity(
	_admin: crate::middleware::auth::AdminGuard,
	State(state): State<AppState>,
	Json(req): Json<CreateActivity>,
) -> Result<Json<Value>, AppError> {
	req.validate()?;
	let item = fitness_service::create_activity(&state.pool, &req).await?;
	Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
	post,
	path = "/api/v1/fitness/ruminations",
	request_body = CreateRumination,
	responses(
		(status = 201, description = "Rumination created", body = serde_json::Value),
		(status = 400, description = "Validation error"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Admin access required")
	),
	security(("cookie_auth" = []))
)]
async fn create_rumination(
	_admin: crate::middleware::auth::AdminGuard,
	State(state): State<AppState>,
	Json(req): Json<CreateRumination>,
) -> Result<Json<Value>, AppError> {
	req.validate()?;
	let item = fitness_service::create_rumination(&state.pool, &req).await?;
	Ok(Json(json!({ "data": item })))
}
