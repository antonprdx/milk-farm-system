use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::fitness::*;

pub async fn list_activities(
    pool: &PgPool,
    filter: &FitnessFilter,
) -> Result<Vec<Activity>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, Activity>(
        "SELECT * FROM activities WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR activity_datetime::date >= $2)
         AND ($3::date IS NULL OR activity_datetime::date <= $3)
         ORDER BY activity_datetime DESC LIMIT $4 OFFSET $5",
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

pub async fn count_activities(pool: &PgPool, filter: &FitnessFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM activities WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR activity_datetime::date >= $2)
         AND ($3::date IS NULL OR activity_datetime::date <= $3)",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn list_ruminations(
    pool: &PgPool,
    filter: &FitnessFilter,
) -> Result<Vec<Rumination>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, Rumination>(
        "SELECT * FROM ruminations WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR date >= $2) AND ($3::date IS NULL OR date <= $3)
         ORDER BY date DESC LIMIT $4 OFFSET $5",
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

pub async fn count_ruminations(pool: &PgPool, filter: &FitnessFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ruminations WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR date >= $2) AND ($3::date IS NULL OR date <= $3)",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create_activity(
    pool: &PgPool,
    req: &CreateActivity,
) -> Result<Activity, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM animals WHERE id = $1 AND active = true)")
            .bind(req.animal_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(AppError::Database)?;
    if !exists {
        return Err(AppError::NotFound(format!(
            "Животное с ID {} не найдено или неактивно",
            req.animal_id
        )));
    }

    let row = sqlx::query_as::<_, Activity>(
        "INSERT INTO activities (animal_id, activity_datetime, activity_counter, heat_attention)
         VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(req.animal_id)
    .bind(req.activity_datetime)
    .bind(req.activity_counter)
    .bind(req.heat_attention)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    tx.commit().await.map_err(AppError::Database)?;
    Ok(row)
}

pub async fn create_rumination(
    pool: &PgPool,
    req: &CreateRumination,
) -> Result<Rumination, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM animals WHERE id = $1 AND active = true)")
            .bind(req.animal_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(AppError::Database)?;
    if !exists {
        return Err(AppError::NotFound(format!(
            "Животное с ID {} не найдено или неактивно",
            req.animal_id
        )));
    }

    let row = sqlx::query_as::<_, Rumination>(
        "INSERT INTO ruminations (animal_id, date, eating_seconds, rumination_minutes)
         VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(req.animal_id)
    .bind(req.date)
    .bind(req.eating_seconds)
    .bind(req.rumination_minutes)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    tx.commit().await.map_err(AppError::Database)?;
    Ok(row)
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
    async fn test_list_activities_empty(pool: PgPool) {
        let filter = FitnessFilter {
            animal_id: None,
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let activities = list_activities(&pool, &filter).await.unwrap();
        assert!(activities.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_ruminations_empty(pool: PgPool) {
        let filter = FitnessFilter {
            animal_id: None,
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let rums = list_ruminations(&pool, &filter).await.unwrap();
        assert!(rums.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_activities_with_data(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        sqlx::query(
            "INSERT INTO activities (animal_id, activity_datetime, activity_counter, heat_attention) VALUES ($1, '2025-01-15T10:00:00Z'::timestamptz, 150, true)"
        )
        .bind(animal_id)
        .execute(&pool)
        .await
        .unwrap();

        let filter = FitnessFilter {
            animal_id: Some(animal_id.to_string()),
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let activities = list_activities(&pool, &filter).await.unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].activity_counter, Some(150));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_ruminations_with_data(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        sqlx::query(
            "INSERT INTO ruminations (animal_id, date, eating_seconds, rumination_minutes) VALUES ($1, '2025-01-15'::date, 18000, 420)"
        )
        .bind(animal_id)
        .execute(&pool)
        .await
        .unwrap();

        let filter = FitnessFilter {
            animal_id: Some(animal_id.to_string()),
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let rums = list_ruminations(&pool, &filter).await.unwrap();
        assert_eq!(rums.len(), 1);
        assert_eq!(rums[0].rumination_minutes, Some(420));
    }
}
