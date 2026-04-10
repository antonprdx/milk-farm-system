use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::grazing::{GrazingData, GrazingFilter};

pub async fn list(pool: &PgPool, filter: &GrazingFilter) -> Result<Vec<GrazingData>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, GrazingData>(
        "SELECT * FROM grazing_data WHERE ($1::date IS NULL OR date >= $1)
         AND ($2::date IS NULL OR date <= $2) ORDER BY date DESC LIMIT $3 OFFSET $4",
    )
    .bind(filter.from_date)
    .bind(filter.till_date)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count(pool: &PgPool, filter: &GrazingFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM grazing_data WHERE ($1::date IS NULL OR date >= $1) AND ($2::date IS NULL OR date <= $2)"
    )
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

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_empty(pool: PgPool) {
        let filter = GrazingFilter { from_date: None, till_date: None, page: None, per_page: None };
        let data = list(&pool, &filter).await.unwrap();
        assert!(data.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_with_data(pool: PgPool) {
        sqlx::query(
            "INSERT INTO grazing_data (date, animal_count, pasture_time) VALUES ('2025-06-15'::date, 50, 360)"
        )
        .execute(&pool)
        .await
        .unwrap();

        let filter = GrazingFilter { from_date: None, till_date: None, page: None, per_page: None };
        let data = list(&pool, &filter).await.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].animal_count, Some(50));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_filter_by_date(pool: PgPool) {
        sqlx::query("INSERT INTO grazing_data (date, animal_count) VALUES ('2025-01-10'::date, 10)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO grazing_data (date, animal_count) VALUES ('2025-06-10'::date, 20)")
            .execute(&pool).await.unwrap();

        let filter = GrazingFilter {
            from_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
            till_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 30).unwrap()),
            page: None,
            per_page: None,
        };
        let data = list(&pool, &filter).await.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].animal_count, Some(20));
    }
}
