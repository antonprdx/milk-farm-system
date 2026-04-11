use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::pagination::paginated;
use crate::models::reproduction::*;
use crate::services::reproduction_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/reproduction/calvings",
            get(list_calvings).post(create_calving),
        )
        .route(
            "/reproduction/calvings/{id}",
            get(get_calving).put(update_calving).delete(delete_calving),
        )
        .route(
            "/reproduction/inseminations",
            get(list_inseminations).post(create_insemination),
        )
        .route(
            "/reproduction/inseminations/{id}",
            get(get_insemination)
                .put(update_insemination)
                .delete(delete_insemination),
        )
        .route(
            "/reproduction/pregnancies",
            get(list_pregnancies).post(create_pregnancy),
        )
        .route(
            "/reproduction/pregnancies/{id}",
            get(get_pregnancy)
                .put(update_pregnancy)
                .delete(delete_pregnancy),
        )
        .route("/reproduction/heats", get(list_heats).post(create_heat))
        .route(
            "/reproduction/heats/{id}",
            get(get_heat).put(update_heat).delete(delete_heat),
        )
        .route(
            "/reproduction/dryoffs",
            get(list_dryoffs).post(create_dryoff),
        )
        .route(
            "/reproduction/dryoffs/{id}",
            get(get_dryoff).put(update_dryoff).delete(delete_dryoff),
        )
        .route("/reproduction/status", get(current_status))
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/calvings",
    responses(
        (status = 200, description = "List of calvings", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(ReproductionFilter),
    security(("cookie_auth" = []))
)]
async fn list_calvings(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(filter.page, filter.per_page, || reproduction_service::list_calvings(pool, f), || reproduction_service::count_calvings(pool, f)).await
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/calvings/{id}",
    responses(
        (status = 200, description = "Calving found", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Calving ID")),
    security(("cookie_auth" = []))
)]
async fn get_calving(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_calving(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Calving {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    post,
    path = "/api/v1/reproduction/calvings",
    request_body = CreateCalving,
    responses(
        (status = 201, description = "Calving created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn create_calving(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateCalving>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_calving(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    put,
    path = "/api/v1/reproduction/calvings/{id}",
    request_body = UpdateCalving,
    responses(
        (status = 200, description = "Calving updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Calving ID")),
    security(("cookie_auth" = []))
)]
async fn update_calving(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateCalving>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_calving(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/reproduction/calvings/{id}",
    responses(
        (status = 200, description = "Calving deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Calving ID")),
    security(("cookie_auth" = []))
)]
async fn delete_calving(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_calving(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/inseminations",
    responses(
        (status = 200, description = "List of inseminations", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(ReproductionFilter),
    security(("cookie_auth" = []))
)]
async fn list_inseminations(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(filter.page, filter.per_page, || reproduction_service::list_inseminations(pool, f), || reproduction_service::count_inseminations(pool, f)).await
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/inseminations/{id}",
    responses(
        (status = 200, description = "Insemination found", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Insemination ID")),
    security(("cookie_auth" = []))
)]
async fn get_insemination(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_insemination(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Insemination {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    post,
    path = "/api/v1/reproduction/inseminations",
    request_body = CreateInsemination,
    responses(
        (status = 201, description = "Insemination created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn create_insemination(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateInsemination>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_insemination(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    put,
    path = "/api/v1/reproduction/inseminations/{id}",
    request_body = UpdateInsemination,
    responses(
        (status = 200, description = "Insemination updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Insemination ID")),
    security(("cookie_auth" = []))
)]
async fn update_insemination(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateInsemination>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_insemination(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/reproduction/inseminations/{id}",
    responses(
        (status = 200, description = "Insemination deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Insemination ID")),
    security(("cookie_auth" = []))
)]
async fn delete_insemination(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_insemination(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/pregnancies",
    responses(
        (status = 200, description = "List of pregnancies", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(ReproductionFilter),
    security(("cookie_auth" = []))
)]
async fn list_pregnancies(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(filter.page, filter.per_page, || reproduction_service::list_pregnancies(pool, f), || reproduction_service::count_pregnancies(pool, f)).await
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/pregnancies/{id}",
    responses(
        (status = 200, description = "Pregnancy found", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Pregnancy ID")),
    security(("cookie_auth" = []))
)]
async fn get_pregnancy(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_pregnancy(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Pregnancy {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    post,
    path = "/api/v1/reproduction/pregnancies",
    request_body = CreatePregnancy,
    responses(
        (status = 201, description = "Pregnancy created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn create_pregnancy(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreatePregnancy>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_pregnancy(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    put,
    path = "/api/v1/reproduction/pregnancies/{id}",
    request_body = UpdatePregnancy,
    responses(
        (status = 200, description = "Pregnancy updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Pregnancy ID")),
    security(("cookie_auth" = []))
)]
async fn update_pregnancy(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdatePregnancy>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_pregnancy(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/reproduction/pregnancies/{id}",
    responses(
        (status = 200, description = "Pregnancy deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Pregnancy ID")),
    security(("cookie_auth" = []))
)]
async fn delete_pregnancy(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_pregnancy(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/heats",
    responses(
        (status = 200, description = "List of heats", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(ReproductionFilter),
    security(("cookie_auth" = []))
)]
async fn list_heats(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(filter.page, filter.per_page, || reproduction_service::list_heats(pool, f), || reproduction_service::count_heats(pool, f)).await
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/heats/{id}",
    responses(
        (status = 200, description = "Heat found", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Heat ID")),
    security(("cookie_auth" = []))
)]
async fn get_heat(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_heat(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Heat {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    post,
    path = "/api/v1/reproduction/heats",
    request_body = CreateHeat,
    responses(
        (status = 201, description = "Heat created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn create_heat(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateHeat>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_heat(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    put,
    path = "/api/v1/reproduction/heats/{id}",
    request_body = UpdateHeat,
    responses(
        (status = 200, description = "Heat updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Heat ID")),
    security(("cookie_auth" = []))
)]
async fn update_heat(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateHeat>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_heat(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/reproduction/heats/{id}",
    responses(
        (status = 200, description = "Heat deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Heat ID")),
    security(("cookie_auth" = []))
)]
async fn delete_heat(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_heat(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/dryoffs",
    responses(
        (status = 200, description = "List of dry-offs", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(ReproductionFilter),
    security(("cookie_auth" = []))
)]
async fn list_dryoffs(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<ReproductionFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(filter.page, filter.per_page, || reproduction_service::list_dryoffs(pool, f), || reproduction_service::count_dryoffs(pool, f)).await
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/dryoffs/{id}",
    responses(
        (status = 200, description = "Dry-off found", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Dry-off ID")),
    security(("cookie_auth" = []))
)]
async fn get_dryoff(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let item = reproduction_service::get_dryoff(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("DryOff {} not found", id)))?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    post,
    path = "/api/v1/reproduction/dryoffs",
    request_body = CreateDryOff,
    responses(
        (status = 201, description = "Dry-off created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn create_dryoff(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateDryOff>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::create_dryoff(&state.pool, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    put,
    path = "/api/v1/reproduction/dryoffs/{id}",
    request_body = UpdateDryOff,
    responses(
        (status = 200, description = "Dry-off updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Dry-off ID")),
    security(("cookie_auth" = []))
)]
async fn update_dryoff(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateDryOff>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let item = reproduction_service::update_dryoff(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": item })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/reproduction/dryoffs/{id}",
    responses(
        (status = 200, description = "Dry-off deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Dry-off ID")),
    security(("cookie_auth" = []))
)]
async fn delete_dryoff(
    _admin: crate::middleware::auth::AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    reproduction_service::delete_dryoff(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Deleted" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/reproduction/status",
    responses(
        (status = 200, description = "Current reproduction status", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn current_status(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let data = reproduction_service::current_status(&state.pool).await?;
    Ok(Json(json!({ "data": data })))
}
