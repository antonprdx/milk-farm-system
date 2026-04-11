use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::pagination::simple_list;
use crate::services::location_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/locations", get(list))
}

#[utoipa::path(
    get,
    path = "/api/v1/locations",
    responses(
        (status = 200, description = "List of locations", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookie_auth" = []))
)]
async fn list(_claims: Claims, State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    simple_list(location_service::list(pool), location_service::count(pool)).await
}
