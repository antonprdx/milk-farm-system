use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::location::{CreateLocation, Location, UpdateLocation};

pub async fn list(pool: &PgPool) -> Result<Vec<Location>, AppError> {
    sqlx::query_as::<_, Location>("SELECT * FROM locations ORDER BY name")
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn count(pool: &PgPool) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM locations")
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create(pool: &PgPool, req: &CreateLocation) -> Result<Location, AppError> {
    sqlx::query_as::<_, Location>(
        "INSERT INTO locations (name, location_type) VALUES ($1, $2) RETURNING *",
    )
    .bind(&req.name)
    .bind(&req.location_type)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update(pool: &PgPool, id: i32, req: &UpdateLocation) -> Result<Location, AppError> {
    sqlx::query_as::<_, Location>(
        "UPDATE locations SET name = COALESCE($2, name),
         location_type = COALESCE($3, location_type)
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.location_type)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("Локация {} не найдена", id)))
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM locations WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Локация {} не найдена",
            id
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_locations_empty(pool: PgPool) {
        let locations = list(&pool).await.unwrap();
        assert!(locations.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_locations_with_data(pool: PgPool) {
        sqlx::query("INSERT INTO locations (name, location_type) VALUES ('Barn A', 'barn')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO locations (name) VALUES ('Pasture 1')")
            .execute(&pool)
            .await
            .unwrap();

        let locations = list(&pool).await.unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].name, "Barn A");
        assert_eq!(locations[1].name, "Pasture 1");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_location(pool: PgPool) {
        let req = CreateLocation {
            name: "New Barn".into(),
            location_type: Some("barn".into()),
        };
        let loc = create(&pool, &req).await.unwrap();
        assert_eq!(loc.name, "New Barn");
        assert_eq!(loc.location_type.as_deref(), Some("barn"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_location(pool: PgPool) {
        sqlx::query("INSERT INTO locations (name, location_type) VALUES ('Old', 'barn')")
            .execute(&pool)
            .await
            .unwrap();

        let req = UpdateLocation {
            name: Some("Updated".into()),
            location_type: Some("pasture".into()),
        };
        let loc = update(&pool, 1, &req).await.unwrap();
        assert_eq!(loc.name, "Updated");
        assert_eq!(loc.location_type.as_deref(), Some("pasture"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_location(pool: PgPool) {
        sqlx::query("INSERT INTO locations (name) VALUES ('ToDelete')")
            .execute(&pool)
            .await
            .unwrap();

        delete(&pool, 1).await.unwrap();
        assert!(list(&pool).await.unwrap().is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_not_found(pool: PgPool) {
        let result = delete(&pool, 999).await;
        assert!(result.is_err());
    }
}
