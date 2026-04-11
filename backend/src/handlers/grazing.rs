use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

use crate::errors::AppError;
use crate::middleware::auth::Claims;
use crate::models::grazing::GrazingFilter;
use crate::models::pagination::paginated;
use crate::services::grazing_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/grazing", get(list_grazing))
}

#[utoipa::path(
    get,
    path = "/api/v1/grazing",
    responses(
        (status = 200, description = "List of grazing data", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    ),
    params(GrazingFilter),
    security(("cookie_auth" = []))
)]
async fn list_grazing(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<GrazingFilter>,
) -> Result<Json<Value>, AppError> {
    let pool = &state.pool;
    let f = &filter;
    paginated(filter.page, filter.per_page, || grazing_service::list(pool, f), || grazing_service::count(pool, f)).await
}
