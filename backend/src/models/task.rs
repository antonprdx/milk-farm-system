use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema, PartialEq)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "task_priority", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "task_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskCategory {
    Health,
    Reproduction,
    Feeding,
    Maintenance,
    Administrative,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub category: TaskCategory,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub animal_id: Option<i32>,
    pub due_date: Option<NaiveDate>,
    pub assigned_to: Option<String>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_by: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    pub category: Option<TaskCategory>,
    pub priority: Option<TaskPriority>,
    pub animal_id: Option<i32>,
    pub due_date: Option<NaiveDate>,
    pub assigned_to: Option<String>,
}

impl CreateTask {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        required_non_empty(&self.title, "Заголовок")?;
        max_len(&self.title, 200, "Заголовок")?;
        opt_max_len(&self.description, 1000, "Описание")?;
        opt_max_len(&self.assigned_to, 100, "Исполнитель")?;
        opt_date_not_future(&self.due_date, "Срок")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<TaskCategory>,
    pub priority: Option<TaskPriority>,
    pub status: Option<TaskStatus>,
    pub animal_id: Option<i32>,
    pub due_date: Option<NaiveDate>,
    pub assigned_to: Option<String>,
}

impl UpdateTask {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if let Some(ref t) = self.title {
            required_non_empty(t, "Заголовок")?;
            max_len(t, 200, "Заголовок")?;
        }
        opt_max_len(&self.description, 1000, "Описание")?;
        opt_max_len(&self.assigned_to, 100, "Исполнитель")?;
        opt_date_not_future(&self.due_date, "Срок")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub category: Option<TaskCategory>,
    pub animal_id: Option<i32>,
    pub overdue: Option<bool>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Transaction {
    pub id: i32,
    pub transaction_type: String,
    pub category: String,
    pub amount: f64,
    pub description: Option<String>,
    pub transaction_date: NaiveDate,
    pub animal_id: Option<i32>,
    pub reference: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTransaction {
    pub transaction_type: String,
    pub category: String,
    pub amount: f64,
    pub description: Option<String>,
    pub transaction_date: NaiveDate,
    pub animal_id: Option<i32>,
    pub reference: Option<String>,
}

impl CreateTransaction {
    pub fn validate(&self) -> Result<(), crate::errors::AppError> {
        use crate::validation::*;
        if self.transaction_type != "income" && self.transaction_type != "expense" {
            return Err(crate::errors::AppError::BadRequest(
                "Тип должен быть income или expense".into(),
            ));
        }
        required_non_empty(&self.category, "Категория")?;
        max_len(&self.category, 100, "Категория")?;
        if self.amount <= 0.0 {
            return Err(crate::errors::AppError::BadRequest(
                "Сумма должна быть положительной".into(),
            ));
        }
        date_not_future(&self.transaction_date, "Дата")?;
        opt_max_len(&self.description, 500, "Описание")?;
        opt_max_len(&self.reference, 100, "Референс")?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct TransactionFilter {
    pub transaction_type: Option<String>,
    pub category: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub till_date: Option<NaiveDate>,
    pub animal_id: Option<i32>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct AuditLogEntry {
    pub id: i32,
    pub user_id: Option<i32>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i32>,
    pub details: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct AuditLogFilter {
    pub user_id: Option<i32>,
    pub entity_type: Option<String>,
    pub action: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
