use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::feed::*;

pub async fn list_day_amounts(
    pool: &PgPool,
    filter: &FeedFilter,
) -> Result<Vec<FeedDayAmount>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, FeedDayAmount>(
        "SELECT * FROM feed_day_amounts WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR feed_date >= $2) AND ($3::date IS NULL OR feed_date <= $3)
         ORDER BY feed_date DESC LIMIT $4 OFFSET $5",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_day_amounts(pool: &PgPool, filter: &FeedFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM feed_day_amounts WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR feed_date >= $2) AND ($3::date IS NULL OR feed_date <= $3)",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn list_visits(pool: &PgPool, filter: &FeedFilter) -> Result<Vec<FeedVisit>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, FeedVisit>(
        "SELECT * FROM feed_visits WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR visit_datetime >= $2) AND ($3::date IS NULL OR visit_datetime <= $3)
         ORDER BY visit_datetime DESC LIMIT $4 OFFSET $5",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_visits(pool: &PgPool, filter: &FeedFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM feed_visits WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR visit_datetime >= $2) AND ($3::date IS NULL OR visit_datetime <= $3)",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn list_types(pool: &PgPool) -> Result<Vec<FeedType>, AppError> {
    sqlx::query_as::<_, FeedType>("SELECT * FROM feed_types ORDER BY number_of_feed_type")
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn count_types(pool: &PgPool) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM feed_types")
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn list_groups(pool: &PgPool) -> Result<Vec<FeedGroup>, AppError> {
    sqlx::query_as::<_, FeedGroup>("SELECT * FROM feed_groups ORDER BY name")
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn count_groups(pool: &PgPool) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM feed_groups")
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn seed_cow(pool: &PgPool) -> i32 {
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO animals (gender, birth_date, active) VALUES ('female', '2020-01-01'::date, true) RETURNING id"
        )
        .fetch_one(pool)
        .await
        .unwrap();
        row.0
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_day_amounts_empty(pool: PgPool) {
        let filter = FeedFilter {
            animal_id: None,
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let amounts = list_day_amounts(&pool, &filter).await.unwrap();
        assert!(amounts.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_visits_empty(pool: PgPool) {
        let filter = FeedFilter {
            animal_id: None,
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let visits = list_visits(&pool, &filter).await.unwrap();
        assert!(visits.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_types_empty(pool: PgPool) {
        let types = list_types(&pool).await.unwrap();
        assert!(types.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_groups_empty(pool: PgPool) {
        let groups = list_groups(&pool).await.unwrap();
        assert!(groups.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_day_amounts_with_data(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        sqlx::query(
            "INSERT INTO feed_day_amounts (animal_id, feed_date, feed_number, total) VALUES ($1, '2025-01-15'::date, 1, 25.0)"
        )
        .bind(animal_id)
        .execute(&pool)
        .await
        .unwrap();

        let filter = FeedFilter {
            animal_id: Some(animal_id.to_string()),
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let amounts = list_day_amounts(&pool, &filter).await.unwrap();
        assert_eq!(amounts.len(), 1);
        assert_eq!(amounts[0].total, 25.0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_visits_with_data(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        sqlx::query(
            "INSERT INTO feed_visits (animal_id, visit_datetime, feed_number, amount, duration_seconds) VALUES ($1, '2025-01-15T10:00:00Z'::timestamptz, 1, 5.0, 300)"
        )
        .bind(animal_id)
        .execute(&pool)
        .await
        .unwrap();

        let filter = FeedFilter {
            animal_id: Some(animal_id.to_string()),
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let visits = list_visits(&pool, &filter).await.unwrap();
        assert_eq!(visits.len(), 1);
    }
}
