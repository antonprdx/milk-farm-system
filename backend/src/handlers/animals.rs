use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, Claims};
use crate::models::animal::{AnimalFilter, CreateAnimal, UpdateAnimal};
use crate::models::GenderType;
use crate::models::animal_stats::AnimalStats;
use crate::models::pagination::{Pagination, paginated};
use crate::models::timeline::{TimelineFilter, TimelineResponse};
use crate::services::{animal_service, animal_stats_service, timeline_service};
use crate::state::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BatchDeactivateRequest {
    pub ids: Vec<i32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CsvImportRequest {
    pub csv: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/animals", get(list).post(create))
        .route("/animals/batch/deactivate", post(batch_deactivate))
        .route("/animals/import/csv", post(import_csv))
        .route("/animals/{id}", get(get_by_id).put(update).delete(remove))
        .route("/animals/{id}/timeline", get(timeline))
        .route("/animals/{id}/stats", get(stats))
        .route("/animals/{id}/export/pdf", get(export_pdf))
}

#[utoipa::path(
    get,
    path = "/api/v1/animals",
    responses(
        (status = 200, description = "List of animals", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(AnimalFilter),
    security(("cookie_auth" = []))
)]
async fn list(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<AnimalFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(
        filter.page,
        filter.per_page,
        || animal_service::list(pool, f),
        || animal_service::count(pool, f),
    )
    .await
}

#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}",
    responses(
        (status = 200, description = "Animal found", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Animal ID")),
    security(("cookie_auth" = []))
)]
async fn get_by_id(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let animal = animal_service::get_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Животное с ID {} не найдено", id)))?;
    Ok(Json(json!({ "data": animal })))
}

#[utoipa::path(
    post,
    path = "/api/v1/animals",
    request_body = CreateAnimal,
    responses(
        (status = 201, description = "Animal created", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn create(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateAnimal>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let animal = animal_service::create(&state.pool, &req).await?;
    Ok(Json(json!({ "data": animal })))
}

#[utoipa::path(
    put,
    path = "/api/v1/animals/{id}",
    request_body = UpdateAnimal,
    responses(
        (status = 200, description = "Animal updated", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Animal ID")),
    security(("cookie_auth" = []))
)]
async fn update(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateAnimal>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let animal = animal_service::update(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": animal })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/animals/{id}",
    responses(
        (status = 200, description = "Animal deleted", body = serde_json::Value),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    params(("id" = i32, Path, description = "Animal ID")),
    security(("cookie_auth" = []))
)]
async fn remove(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    animal_service::delete(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}

#[utoipa::path(
    post,
    path = "/api/v1/animals/batch/deactivate",
    request_body = BatchDeactivateRequest,
    responses(
        (status = 200, description = "Animals deactivated", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn batch_deactivate(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<BatchDeactivateRequest>,
) -> Result<Json<Value>, AppError> {
    if req.ids.is_empty() {
        return Err(AppError::BadRequest("Список ID пуст".into()));
    }
    if req.ids.len() > 500 {
        return Err(AppError::BadRequest(
            "Максимум 500 записей за один запрос".into(),
        ));
    }
    let count = animal_service::batch_deactivate(&state.pool, &req.ids).await?;
    Ok(Json(json!({ "message": format!("Деактивировано {} животных", count), "count": count })))
}

#[utoipa::path(
    post,
    path = "/api/v1/animals/import/csv",
    request_body = CsvImportRequest,
    responses(
        (status = 200, description = "Animals imported", body = serde_json::Value),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Admin access required")
    ),
    security(("cookie_auth" = []))
)]
async fn import_csv(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CsvImportRequest>,
) -> Result<Json<Value>, AppError> {
    if req.csv.trim().is_empty() {
        return Err(AppError::BadRequest("CSV пуст".into()));
    }
    let mut created = 0u32;
    let mut errors: Vec<String> = Vec::new();
    for (i, line) in req.csv.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 2 {
            errors.push(format!("Строка {}: недостаточно столбцов", i + 1));
            continue;
        }
        let gender = cols[0].trim().to_lowercase();
        if gender != "male" && gender != "female" && gender != "м" && gender != "ж" {
            errors.push(format!("Строка {}: пол должен быть male/female (или м/ж)", i + 1));
            continue;
        }
        let gender = if gender == "м" { "male" } else if gender == "ж" { "female" } else { &gender };
        let birth_date_str = cols[1].trim();
        let birth_date = match chrono::NaiveDate::parse_from_str(birth_date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                errors.push(format!("Строка {}: неверная дата '{}', формат YYYY-MM-DD", i + 1, birth_date_str));
                continue;
            }
        };
        let name = cols.get(2).map(|s| s.trim()).filter(|s| !s.is_empty()).map(String::from);
        let life_number = cols.get(3).map(|s| s.trim()).filter(|s| !s.is_empty()).map(String::from);
        let gender_val = match gender {
            "male" => GenderType::Male,
            "female" => GenderType::Female,
            _ => unreachable!(),
        };
        let animal = animal_service::create(
            &state.pool,
            &crate::models::animal::CreateAnimal {
                gender: gender_val,
                birth_date,
                name,
                life_number,
                user_number: None,
                hair_color_code: None,
                father_life_number: None,
                mother_life_number: None,
                description: None,
                ucn_number: None,
                use_as_sire: None,
                location: None,
                group_number: None,
                keep: None,
                gestation: None,
                responder_number: None,
            },
        )
        .await;
        match animal {
            Ok(_) => created += 1,
            Err(e) => errors.push(format!("Строка {}: {}", i + 1, e)),
        }
    }
    Ok(Json(json!({
        "created": created,
        "errors": errors,
        "total": created + errors.len() as u32,
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}/timeline",
    responses(
        (status = 200, description = "Timeline events", body = TimelineResponse),
        (status = 404, description = "Animal not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(
        ("id" = i32, Path, description = "Animal ID"),
        TimelineFilter
    ),
    security(("cookie_auth" = []))
)]
async fn timeline(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Query(filter): Query<TimelineFilter>,
) -> Result<Json<TimelineResponse>, AppError> {
    animal_service::ensure_exists(&state.pool, id).await?;
    let pag = Pagination::from_filter(filter.page, filter.per_page);
    let (data, total) = tokio::join!(
        timeline_service::list(&state.pool, id, filter.page, filter.per_page),
        timeline_service::count(&state.pool, id),
    );
    let data = data?;
    let total = total?;
    Ok(Json(TimelineResponse {
        data,
        total,
        page: pag.page,
        per_page: pag.per_page,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}/stats",
    responses(
        (status = 200, description = "Animal statistics", body = AnimalStats),
        (status = 404, description = "Animal not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Animal ID")),
    security(("cookie_auth" = []))
)]
async fn stats(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<AnimalStats>, AppError> {
    animal_service::ensure_exists(&state.pool, id).await?;
    let stats = animal_stats_service::get_animal_stats(&state.pool, id).await?;
    Ok(Json(stats))
}

#[utoipa::path(
    get,
    path = "/api/v1/animals/{id}/export/pdf",
    responses(
        (status = 200, description = "Animal card PDF"),
        (status = 404, description = "Animal not found"),
        (status = 401, description = "Unauthorized")
    ),
    params(("id" = i32, Path, description = "Animal ID")),
    security(("cookie_auth" = []))
)]
async fn export_pdf(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Response, AppError> {
    let pdf = crate::services::pdf_service::generate_animal_card(&state.pool, id).await?;
    let filename = format!("animal_{}.pdf", id);
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        pdf,
    )
        .into_response())
}
