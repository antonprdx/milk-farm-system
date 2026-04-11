use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::user::User;
use crate::services::retry::retry_db;

pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<User>, AppError> {
    let pool = pool.clone();
    let username = username.to_string();
    retry_db(move || {
        let pool = pool.clone();
        let username = username.clone();
        async move {
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
                .bind(&username)
                .fetch_optional(&pool)
                .await
                .map_err(AppError::Database)
        }
    }).await
}

pub async fn find_by_id(pool: &PgPool, user_id: i32) -> Result<Option<User>, AppError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<User>, AppError> {
    sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
    role: &str,
) -> Result<User, AppError> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password_hash, role) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(username)
    .bind(password_hash)
    .bind(role)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_password(pool: &PgPool, user_id: i32, new_hash: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn update_password_and_clear_flag(
    pool: &PgPool,
    user_id: i32,
    new_hash: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET password_hash = $1, must_change_password = false WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(())
}

pub async fn delete_user(pool: &PgPool, user_id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".into()));
    }
    Ok(())
}

pub async fn update_role(pool: &PgPool, user_id: i32, role: &str) -> Result<(), AppError> {
    let result = sqlx::query("UPDATE users SET role = $1 WHERE id = $2")
        .bind(role)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_user(pool: PgPool) {
        let user = create_user(&pool, "testuser", "hash123", "user")
            .await
            .unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.role, "user");
        assert_eq!(user.password_hash, "hash123");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_duplicate_user_fails(pool: PgPool) {
        create_user(&pool, "dup", "hash", "user").await.unwrap();
        let result = create_user(&pool, "dup", "hash2", "user").await;
        assert!(result.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_username_exists(pool: PgPool) {
        create_user(&pool, "findme", "hash", "admin").await.unwrap();
        let found = find_by_username(&pool, "findme").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().username, "findme");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_find_by_username_not_exists(pool: PgPool) {
        let found = find_by_username(&pool, "nobody").await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_users_empty(pool: PgPool) {
        let users = list_users(&pool).await.unwrap();
        assert_eq!(users.len(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_users_multiple(pool: PgPool) {
        create_user(&pool, "user1", "h1", "admin").await.unwrap();
        create_user(&pool, "user2", "h2", "user").await.unwrap();
        let users = list_users(&pool).await.unwrap();
        assert_eq!(users.len(), 3);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_password(pool: PgPool) {
        let user = create_user(&pool, "pwtest", "old_hash", "user")
            .await
            .unwrap();
        update_password(&pool, user.id, "new_hash").await.unwrap();
        let updated = find_by_username(&pool, "pwtest").await.unwrap().unwrap();
        assert_eq!(updated.password_hash, "new_hash");
    }
}
