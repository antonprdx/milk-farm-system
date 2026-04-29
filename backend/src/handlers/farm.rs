use axum::extract::State;
use axum::response::Json;
use serde::Deserialize;
use serde_json::json;

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, Claims};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ZoneReq {
    pub name: String,
    pub zone_type: String,
    pub capacity: Option<i32>,
    pub position_data: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateZoneReq {
    pub name: Option<String>,
    pub zone_type: Option<String>,
    pub capacity: Option<i32>,
    pub position_data: Option<serde_json::Value>,
}

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/farm/zones", axum::routing::get(list_zones).post(create_zone))
        .route("/farm/zones/{id}", axum::routing::put(update_zone).delete(delete_zone))
}

async fn list_zones(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows: Vec<(i32, String, String, Option<i32>, serde_json::Value, String)> = sqlx::query_as(
        "SELECT id, name, zone_type::text, capacity, COALESCE(position_data, '{}'), TO_CHAR(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') FROM farm_zones ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::Database)?;

    let zones: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, name, zt, cap, pos, ca)| json!({ "id": id, "name": name, "zone_type": zt, "capacity": cap, "position_data": pos, "created_at": ca }))
        .collect();
    Ok(Json(json!({ "data": zones })))
}

async fn create_zone(
    _claims: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<ZoneReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let row = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO farm_zones (name, zone_type, capacity, position_data) VALUES ($1, $2::varchar, $3, $4) RETURNING id",
    )
    .bind(&req.name)
    .bind(&req.zone_type)
    .bind(req.capacity)
    .bind(req.position_data.unwrap_or(json!({})))
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(json!({ "data": { "id": row.0 } })))
}

async fn update_zone(
    _claims: AdminGuard,
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i32>,
    Json(req): Json<UpdateZoneReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query(
        "UPDATE farm_zones SET name = COALESCE($2, name), zone_type = COALESCE($3, zone_type), capacity = COALESCE($4, capacity), position_data = COALESCE($5, position_data) WHERE id = $1",
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.zone_type)
    .bind(req.capacity)
    .bind(&req.position_data)
    .execute(&state.pool)
    .await
    .map_err(AppError::Database)?;
    Ok(Json(json!({ "message": "Обновлено" })))
}

async fn delete_zone(
    _claims: AdminGuard,
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM farm_zones WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Zone not found".into()));
    }
    Ok(Json(json!({ "message": "Удалено" })))
}


