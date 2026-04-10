use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::location::Location;

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
}
