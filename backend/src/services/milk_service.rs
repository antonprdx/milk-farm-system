use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::milk::*;
use crate::services::animal_service;

pub async fn list_productions(
    pool: &PgPool,
    filter: &MilkFilter,
) -> Result<Vec<MilkDayProduction>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, MilkDayProduction>(
        "SELECT * FROM milk_day_productions WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::date IS NULL OR date >= $2) AND ($3::date IS NULL OR date <= $3)
         ORDER BY date DESC LIMIT $4 OFFSET $5",
    )
    .bind(filter.animal_id)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_productions(pool: &PgPool, filter: &MilkFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM milk_day_productions WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::date IS NULL OR date >= $2) AND ($3::date IS NULL OR date <= $3)",
    )
    .bind(filter.animal_id)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn get_production(pool: &PgPool, id: i32) -> Result<Option<MilkDayProduction>, AppError> {
    sqlx::query_as::<_, MilkDayProduction>("SELECT * FROM milk_day_productions WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create_production(
    pool: &PgPool,
    req: &CreateMilkDayProduction,
) -> Result<MilkDayProduction, AppError> {
    animal_service::ensure_exists(pool, req.animal_id).await?;

    sqlx::query_as::<_, MilkDayProduction>(
        "INSERT INTO milk_day_productions (animal_id, date, milk_amount, avg_amount, avg_weight, isk)
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING *",
    )
    .bind(req.animal_id)
    .bind(req.date)
    .bind(req.milk_amount)
    .bind(req.avg_amount)
    .bind(req.avg_weight)
    .bind(req.isk)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_production(
    pool: &PgPool,
    id: i32,
    req: &UpdateMilkDayProduction,
) -> Result<MilkDayProduction, AppError> {
    sqlx::query_as::<_, MilkDayProduction>(
        "UPDATE milk_day_productions SET
         date = COALESCE($2, date),
         milk_amount = COALESCE($3, milk_amount),
         avg_amount = COALESCE($4, avg_amount),
         avg_weight = COALESCE($5, avg_weight),
         isk = COALESCE($6, isk)
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(req.date)
    .bind(req.milk_amount)
    .bind(req.avg_amount)
    .bind(req.avg_weight)
    .bind(req.isk)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_production(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM milk_day_productions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Production {} not found", id)));
    }
    Ok(())
}

pub async fn list_visits(pool: &PgPool, filter: &MilkFilter) -> Result<Vec<MilkVisit>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, MilkVisit>(
        "SELECT * FROM milk_visits WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::date IS NULL OR visit_datetime >= $2) AND ($3::date IS NULL OR visit_datetime <= $3)
         ORDER BY visit_datetime DESC LIMIT $4 OFFSET $5",
    )
    .bind(filter.animal_id)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_visits(pool: &PgPool, filter: &MilkFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM milk_visits WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::date IS NULL OR visit_datetime >= $2) AND ($3::date IS NULL OR visit_datetime <= $3)",
    )
    .bind(filter.animal_id)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn list_quality(
    pool: &PgPool,
    filter: &MilkFilter,
) -> Result<Vec<MilkQuality>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, MilkQuality>(
        "SELECT * FROM milk_quality WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::date IS NULL OR date >= $2) AND ($3::date IS NULL OR date <= $3)
         ORDER BY date DESC LIMIT $4 OFFSET $5",
    )
    .bind(filter.animal_id)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count_quality(pool: &PgPool, filter: &MilkFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM milk_quality WHERE ($1::int IS NULL OR animal_id = $1)
         AND ($2::date IS NULL OR date >= $2) AND ($3::date IS NULL OR date <= $3)",
    )
    .bind(filter.animal_id)
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn seed_cow(pool: &PgPool) -> i32 {
        let animal = sqlx::query_as::<_, crate::models::animal::Animal>(
            "INSERT INTO animals (gender, birth_date, active) VALUES ('female', '2020-01-01'::date, true) RETURNING *"
        )
        .fetch_one(pool)
        .await
        .unwrap();
        animal.id
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_production(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req = CreateMilkDayProduction {
            animal_id,
            date: chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            milk_amount: Some(25.5),
            avg_amount: Some(24.0),
            avg_weight: None,
            isk: None,
        };
        let prod = create_production(&pool, &req).await.unwrap();
        assert_eq!(prod.animal_id, animal_id);
        assert_eq!(prod.milk_amount, Some(25.5));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_production(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req = CreateMilkDayProduction {
            animal_id,
            date: chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            milk_amount: Some(30.0),
            avg_amount: None,
            avg_weight: None,
            isk: None,
        };
        let created = create_production(&pool, &req).await.unwrap();
        let found = get_production(&pool, created.id).await.unwrap();
        assert!(found.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_production_not_found(pool: PgPool) {
        let found = get_production(&pool, 99999).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_productions_empty(pool: PgPool) {
        let filter = MilkFilter {
            animal_id: None,
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let prods = list_productions(&pool, &filter).await.unwrap();
        assert!(prods.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_productions_filter_by_animal(pool: PgPool) {
        let a1 = seed_cow(&pool).await;
        let a2 = seed_cow(&pool).await;
        let req1 = CreateMilkDayProduction {
            animal_id: a1,
            date: chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            milk_amount: Some(10.0),
            avg_amount: None,
            avg_weight: None,
            isk: None,
        };
        let req2 = CreateMilkDayProduction {
            animal_id: a2,
            date: chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            milk_amount: Some(20.0),
            avg_amount: None,
            avg_weight: None,
            isk: None,
        };
        create_production(&pool, &req1).await.unwrap();
        create_production(&pool, &req2).await.unwrap();
        let filter = MilkFilter {
            animal_id: Some(a1),
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let prods = list_productions(&pool, &filter).await.unwrap();
        assert_eq!(prods.len(), 1);
        assert_eq!(prods[0].animal_id, a1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_productions_filter_by_date(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req1 = CreateMilkDayProduction {
            animal_id,
            date: chrono::NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            milk_amount: Some(10.0),
            avg_amount: None,
            avg_weight: None,
            isk: None,
        };
        let req2 = CreateMilkDayProduction {
            animal_id,
            date: chrono::NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),
            milk_amount: Some(20.0),
            avg_amount: None,
            avg_weight: None,
            isk: None,
        };
        create_production(&pool, &req1).await.unwrap();
        create_production(&pool, &req2).await.unwrap();
        let filter = MilkFilter {
            animal_id: None,
            from_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
            till_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 25).unwrap()),
            page: None,
            per_page: None,
        };
        let prods = list_productions(&pool, &filter).await.unwrap();
        assert_eq!(prods.len(), 1);
        assert_eq!(prods[0].milk_amount, Some(20.0));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_visits_empty(pool: PgPool) {
        let filter = MilkFilter {
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
    async fn test_list_quality_empty(pool: PgPool) {
        let filter = MilkFilter {
            animal_id: None,
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let quality = list_quality(&pool, &filter).await.unwrap();
        assert!(quality.is_empty());
    }
}
