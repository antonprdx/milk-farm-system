use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::services::location_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/locations", get(list))
}

async fn list(_claims: Claims, State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let data = location_service::list(&state.pool).await?;
    let total = location_service::count(&state.pool).await?;
    Ok(Json(json!({ "data": data, "total": total, "page": 1, "per_page": total })))
}
