use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::task::{
    AuditLogEntry, AuditLogFilter, CreateTask, CreateTransaction, Task, TaskCategory, TaskFilter,
    TaskPriority, TaskStatus, Transaction, TransactionFilter,
};

pub async fn list_tasks(pool: &PgPool, filter: &TaskFilter) -> Result<Vec<Task>, AppError> {
    let pag = crate::models::pagination::Pagination::new(filter.page, filter.per_page, 20, 100);
    let overdue = filter.overdue.unwrap_or(false);
    sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks
         WHERE ($1::task_status IS NULL OR status = $1)
         AND ($2::task_priority IS NULL OR priority = $2)
         AND ($3::task_category IS NULL OR category = $3)
         AND ($4::int IS NULL OR animal_id = $4)
         AND (NOT $5 OR (due_date IS NOT NULL AND due_date < CURRENT_DATE AND status NOT IN ('done','cancelled')))
         ORDER BY CASE priority WHEN 'urgent'::task_priority THEN 0 WHEN 'high'::task_priority THEN 1 WHEN 'medium'::task_priority THEN 2 ELSE 3 END, due_date ASC NULLS LAST, id DESC
         LIMIT $6 OFFSET $7",
    )
    .bind(&filter.status)
    .bind(&filter.priority)
    .bind(&filter.category)
    .bind(filter.animal_id)
    .bind(overdue)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_tasks(pool: &PgPool, filter: &TaskFilter) -> Result<i64, AppError> {
    let overdue = filter.overdue.unwrap_or(false);
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tasks
         WHERE ($1::task_status IS NULL OR status = $1)
         AND ($2::task_priority IS NULL OR priority = $2)
         AND ($3::task_category IS NULL OR category = $3)
         AND ($4::int IS NULL OR animal_id = $4)
         AND (NOT $5 OR (due_date IS NOT NULL AND due_date < CURRENT_DATE AND status NOT IN ('done','cancelled')))",
    )
    .bind(&filter.status)
    .bind(&filter.priority)
    .bind(&filter.category)
    .bind(filter.animal_id)
    .bind(overdue)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create_task(pool: &PgPool, req: &CreateTask, user_id: Option<i32>) -> Result<Task, AppError> {
    if let Some(aid) = req.animal_id {
        crate::services::animal_service::ensure_exists(pool, aid).await?;
    }
    let category = req.category.as_ref().cloned().unwrap_or(TaskCategory::Other);
    let priority = req.priority.as_ref().cloned().unwrap_or(TaskPriority::Medium);
    sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (title, description, category, priority, animal_id, due_date, assigned_to, created_by)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING *",
    )
    .bind(&req.title)
    .bind(&req.description)
    .bind(&category)
    .bind(&priority)
    .bind(req.animal_id)
    .bind(req.due_date)
    .bind(&req.assigned_to)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_task(pool: &PgPool, id: i32, req: &crate::models::task::UpdateTask) -> Result<Task, AppError> {
    let existing = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Задача с ID {} не найдена", id)))?;

    let new_status = req.status.as_ref().cloned().unwrap_or(existing.status);
    let completed_at = match (&new_status, &existing.completed_at) {
        (TaskStatus::Done, None) => Some(chrono::Utc::now()),
        (s, _) if *s != TaskStatus::Done => None,
        (_, ca) => ca.clone(),
    };

    sqlx::query_as::<_, Task>(
        "UPDATE tasks SET
         title = COALESCE($2, title),
         description = COALESCE($3, description),
         category = COALESCE($4, category),
         priority = COALESCE($5, priority),
         status = $6,
         animal_id = COALESCE($7, animal_id),
         due_date = COALESCE($8, due_date),
         assigned_to = COALESCE($9, assigned_to),
         completed_at = $10,
         updated_at = NOW()
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.category)
    .bind(&req.priority)
    .bind(&new_status)
    .bind(req.animal_id)
    .bind(req.due_date)
    .bind(&req.assigned_to)
    .bind(completed_at)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_task(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Задача с ID {} не найдена", id)));
    }
    Ok(())
}

pub async fn list_transactions(pool: &PgPool, filter: &TransactionFilter) -> Result<Vec<Transaction>, AppError> {
    let pag = crate::models::pagination::Pagination::new(filter.page, filter.per_page, 20, 100);
    sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions
         WHERE ($1::text IS NULL OR transaction_type = $1)
         AND ($2::text IS NULL OR category ILIKE '%' || $2 || '%')
         AND ($3::date IS NULL OR transaction_date >= $3)
         AND ($4::date IS NULL OR transaction_date <= $4)
         AND ($5::int IS NULL OR animal_id = $5)
         ORDER BY transaction_date DESC, id DESC
         LIMIT $6 OFFSET $7",
    )
    .bind(&filter.transaction_type)
    .bind(&filter.category)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(filter.animal_id)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_transactions(pool: &PgPool, filter: &TransactionFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM transactions
         WHERE ($1::text IS NULL OR transaction_type = $1)
         AND ($2::text IS NULL OR category ILIKE '%' || $2 || '%')
         AND ($3::date IS NULL OR transaction_date >= $3)
         AND ($4::date IS NULL OR transaction_date <= $4)
         AND ($5::int IS NULL OR animal_id = $5)",
    )
    .bind(&filter.transaction_type)
    .bind(&filter.category)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(filter.animal_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create_transaction(pool: &PgPool, req: &CreateTransaction) -> Result<Transaction, AppError> {
    if let Some(aid) = req.animal_id {
        crate::services::animal_service::ensure_exists(pool, aid).await?;
    }
    sqlx::query_as::<_, Transaction>(
        "INSERT INTO transactions (transaction_type, category, amount, description, transaction_date, animal_id, reference)
         VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING *",
    )
    .bind(&req.transaction_type)
    .bind(&req.category)
    .bind(req.amount)
    .bind(&req.description)
    .bind(req.transaction_date)
    .bind(req.animal_id)
    .bind(&req.reference)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_transaction(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM transactions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Транзакция с ID {} не найдена", id)));
    }
    Ok(())
}

pub async fn log_audit(
    pool: &PgPool,
    user_id: Option<i32>,
    action: &str,
    entity_type: &str,
    entity_id: Option<i32>,
    details: Option<serde_json::Value>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO audit_log (user_id, action, entity_type, entity_id, details) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(details)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(())
}

pub async fn list_audit_log(pool: &PgPool, filter: &AuditLogFilter) -> Result<Vec<AuditLogEntry>, AppError> {
    let pag = crate::models::pagination::Pagination::new(filter.page, filter.per_page, 20, 100);
    sqlx::query_as::<_, AuditLogEntry>(
        "SELECT * FROM audit_log
         WHERE ($1::int IS NULL OR user_id = $1)
         AND ($2::text IS NULL OR entity_type = $2)
         AND ($3::text IS NULL OR action = $3)
         AND ($4::date IS NULL OR created_at::date >= $4)
         ORDER BY created_at DESC
         LIMIT $5 OFFSET $6",
    )
    .bind(filter.user_id)
    .bind(&filter.entity_type)
    .bind(&filter.action)
    .bind(filter.from_date)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_audit_log(pool: &PgPool, filter: &AuditLogFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log
         WHERE ($1::int IS NULL OR user_id = $1)
         AND ($2::text IS NULL OR entity_type = $2)
         AND ($3::text IS NULL OR action = $3)
         AND ($4::date IS NULL OR created_at::date >= $4)",
    )
    .bind(filter.user_id)
    .bind(&filter.entity_type)
    .bind(&filter.action)
    .bind(filter.from_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn global_search(pool: &PgPool, query: &str) -> Result<serde_json::Value, AppError> {
    let pattern = format!("%{}%", query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
    let animals: Vec<serde_json::Value> = sqlx::query_as::<_, (i32, Option<String>, Option<String>)>(
        "SELECT id, name, life_number FROM animals WHERE active = true AND (name ILIKE $1 OR life_number ILIKE $1 OR CAST(id AS TEXT) ILIKE $1) ORDER BY id LIMIT 10",
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?
    .into_iter()
    .map(|(id, name, ln)| serde_json::json!({"id": id, "name": name, "life_number": ln}))
    .collect();

    let contacts: Vec<serde_json::Value> = sqlx::query_as::<_, (i32, String, Option<String>)>(
        "SELECT id, name, company_name FROM contacts WHERE active = true AND (name ILIKE $1 OR company_name ILIKE $1) ORDER BY id LIMIT 10",
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?
    .into_iter()
    .map(|(id, name, company)| serde_json::json!({"id": id, "name": name, "company_name": company}))
    .collect();

    Ok(serde_json::json!({"animals": animals, "contacts": contacts}))
}
