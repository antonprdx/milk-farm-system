use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::middleware::auth::Claims;
use crate::models::grazing::GrazingFilter;
use crate::services::grazing_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/grazing", get(list_grazing))
}

async fn list_grazing(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<GrazingFilter>,
) -> Result<Json<Value>, crate::errors::AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    let data = grazing_service::list(&state.pool, &filter).await?;
    let total = grazing_service::count(&state.pool, &filter).await?;
    Ok(Json(
        json!({ "data": data, "total": total, "page": pag.page, "per_page": pag.per_page }),
    ))
}
