use axum::extract::{Path, Query, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, Claims};
use crate::models::pagination::paginated;
use crate::models::task::{
    AuditLogFilter, CreateTask, CreateTransaction, TaskFilter, TransactionFilter, UpdateTask,
};
use crate::services::task_service;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task).put(update_task).delete(delete_task))
        .route("/finance", get(list_transactions).post(create_transaction))
        .route("/finance/{id}", delete(delete_transaction))
        .route("/audit-log", get(list_audit))
        .route("/search", get(global_search))
}

async fn list_tasks(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<TaskFilter>,
) -> Result<Json<Value>, AppError> {
    paginated(
        filter.page,
        filter.per_page,
        || task_service::list_tasks(&state.pool, &filter),
        || task_service::count_tasks(&state.pool, &filter),
    )
    .await
}

async fn get_task(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let task = sqlx::query_as::<_, crate::models::task::Task>("SELECT * FROM tasks WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Задача с ID {} не найдена", id)))?;
    Ok(Json(json!({ "data": task })))
}

async fn create_task(
    _claims: Claims,
    State(state): State<AppState>,
    Json(req): Json<CreateTask>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let task = task_service::create_task(&state.pool, &req, None).await?;
    Ok(Json(json!({ "data": task })))
}

async fn update_task(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateTask>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let task = task_service::update_task(&state.pool, id, &req).await?;
    Ok(Json(json!({ "data": task })))
}

async fn delete_task(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    task_service::delete_task(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}

async fn list_transactions(
    _claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<TransactionFilter>,
) -> Result<Json<Value>, AppError> {
    paginated(
        filter.page,
        filter.per_page,
        || task_service::list_transactions(&state.pool, &filter),
        || task_service::count_transactions(&state.pool, &filter),
    )
    .await
}

async fn create_transaction(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<CreateTransaction>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let tx = task_service::create_transaction(&state.pool, &req).await?;
    Ok(Json(json!({ "data": tx })))
}

async fn delete_transaction(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    task_service::delete_transaction(&state.pool, id).await?;
    Ok(Json(json!({ "message": "Удалено" })))
}

async fn list_audit(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Query(filter): Query<AuditLogFilter>,
) -> Result<Json<Value>, AppError> {
    paginated(
        filter.page,
        filter.per_page,
        || task_service::list_audit_log(&state.pool, &filter),
        || task_service::count_audit_log(&state.pool, &filter),
    )
    .await
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn global_search(
    _claims: Claims,
    State(state): State<AppState>,
    Query(sq): Query<SearchQuery>,
) -> Result<Json<Value>, AppError> {
    if sq.q.trim().is_empty() {
        return Ok(Json(json!({"animals": [], "contacts": []})));
    }
    let results = task_service::global_search(&state.pool, &sq.q).await?;
    Ok(Json(results))
}
