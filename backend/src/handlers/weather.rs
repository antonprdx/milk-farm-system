use axum::extract::State;
use axum::response::Json;
use serde_json::json;

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::state::AppState;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/weather", axum::routing::get(get_weather))
        .route("/weather/forecast", axum::routing::get(get_forecast))
}

async fn get_weather(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let current = crate::services::weather_service::get_current_weather(&state.pool).await?;
    Ok(Json(json!({ "data": current })))
}

async fn get_forecast(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let forecast = crate::services::weather_service::get_forecast(&state.pool).await?;
    Ok(Json(json!({ "data": forecast })))
}
