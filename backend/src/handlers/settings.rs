use axum::extract::{State, Path};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::errors::AppError;
use crate::middleware::auth::{AdminGuard, ClaimsAllowMustChange, RegisterRequest};
use crate::models::user::UserPublic;
use crate::services::user_service;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

impl ChangePasswordRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        crate::validation::required_non_empty(&self.old_password, "Старый пароль")?;
        crate::validation::password(&self.new_password)?;
        Ok(())
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/settings/users", get(list_users).post(create_user))
        .route("/settings/users/{id}", delete(delete_user))
        .route("/settings/users/{id}/role", put(update_role))
        .route("/settings/password", post(change_password))
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

async fn list_users(
    _admin: AdminGuard,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let users = user_service::list_users(&state.pool).await?;
    let public_users: Vec<UserPublic> = users.into_iter().map(UserPublic::from).collect();
    Ok(Json(json!({ "data": public_users })))
}

async fn create_user(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;
    user_service::create_user(&state.pool, &req.username, &hash, "user").await?;
    Ok(Json(json!({ "message": "User created" })))
}

async fn change_password(
    claims: ClaimsAllowMustChange,
    State(state): State<AppState>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, AppError> {
    req.validate()?;
    let user = user_service::find_by_username(&state.pool, &claims.0.sub)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;
    let valid = bcrypt::verify(&req.old_password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.into()))?;
    if !valid {
        return Err(AppError::Unauthorized("Invalid old password".into()));
    }
    let new_hash = bcrypt::hash(&req.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;
    user_service::update_password_and_clear_flag(&state.pool, user.id, &new_hash).await?;
    Ok(Json(json!({ "message": "Password changed" })))
}

async fn delete_user(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    user_service::delete_user(&state.pool, id).await?;
    Ok(Json(json!({ "message": "User deleted" })))
}

async fn update_role(
    _admin: AdminGuard,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<UpdateRoleRequest>,
) -> Result<Json<Value>, AppError> {
    if body.role != "admin" && body.role != "user" {
        return Err(AppError::BadRequest("Role must be 'admin' or 'user'".into()));
    }
    user_service::update_role(&state.pool, id, &body.role).await?;
    Ok(Json(json!({ "message": "Role updated" })))
}
