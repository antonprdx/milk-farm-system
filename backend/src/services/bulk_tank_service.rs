use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::bulk_tank::{BulkTankFilter, BulkTankTest, CreateBulkTankTest, UpdateBulkTankTest};

pub async fn list(pool: &PgPool, filter: &BulkTankFilter) -> Result<Vec<BulkTankTest>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);
    sqlx::query_as::<_, BulkTankTest>(
        "SELECT * FROM bulk_tank_tests WHERE ($1::date IS NULL OR date >= $1)
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

pub async fn count(pool: &PgPool, filter: &BulkTankFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bulk_tank_tests WHERE ($1::date IS NULL OR date >= $1)
         AND ($2::date IS NULL OR date <= $2)",
    )
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<BulkTankTest>, AppError> {
    sqlx::query_as::<_, BulkTankTest>("SELECT * FROM bulk_tank_tests WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create(pool: &PgPool, req: &CreateBulkTankTest) -> Result<BulkTankTest, AppError> {
    sqlx::query_as::<_, BulkTankTest>(
        "INSERT INTO bulk_tank_tests (date, fat, protein, lactose, scc, ffa)
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING *",
    )
    .bind(req.date)
    .bind(req.fat)
    .bind(req.protein)
    .bind(req.lactose)
    .bind(req.scc)
    .bind(req.ffa)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update(pool: &PgPool, id: i32, req: &UpdateBulkTankTest) -> Result<BulkTankTest, AppError> {
    let lactose_set = req.lactose.is_some();
    let lactose_val = req.lactose.as_ref().and_then(|v| *v);
    let scc_set = req.scc.is_some();
    let scc_val = req.scc.as_ref().and_then(|v| *v);
    let ffa_set = req.ffa.is_some();
    let ffa_val = req.ffa.as_ref().and_then(|v| *v);

    sqlx::query_as::<_, BulkTankTest>(
        "UPDATE bulk_tank_tests SET
         date = COALESCE($2, date),
         fat = COALESCE($3, fat),
         protein = COALESCE($4, protein),
         lactose = CASE WHEN $5 THEN $6 ELSE lactose END,
         scc = CASE WHEN $7 THEN $8 ELSE scc END,
         ffa = CASE WHEN $9 THEN $10 ELSE ffa END
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(req.date)
    .bind(req.fat)
    .bind(req.protein)
    .bind(lactose_set)
    .bind(lactose_val)
    .bind(scc_set)
    .bind(scc_val)
    .bind(ffa_set)
    .bind(ffa_val)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM bulk_tank_tests WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("BulkTankTest {} not found", id)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_req(date: &str) -> CreateBulkTankTest {
        CreateBulkTankTest {
            date: chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            fat: 3.8,
            protein: 3.2,
            lactose: Some(4.6),
            scc: Some(150),
            ffa: None,
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_bulk_tank(pool: PgPool) {
        let tank = create(&pool, &create_req("2025-01-15")).await.unwrap();
        assert_eq!(tank.fat, 3.8);
        assert_eq!(tank.protein, 3.2);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_empty(pool: PgPool) {
        let filter = BulkTankFilter { from_date: None, till_date: None, page: None, per_page: None };
        let tanks = list(&pool, &filter).await.unwrap();
        assert!(tanks.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_ordered_by_date(pool: PgPool) {
        create(&pool, &create_req("2025-01-10")).await.unwrap();
        create(&pool, &create_req("2025-01-20")).await.unwrap();
        let filter = BulkTankFilter { from_date: None, till_date: None, page: None, per_page: None };
        let tanks = list(&pool, &filter).await.unwrap();
        assert_eq!(tanks.len(), 2);
        assert!(tanks[0].date > tanks[1].date);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_by_id(pool: PgPool) {
        let created = create(&pool, &create_req("2025-01-15")).await.unwrap();
        let found = get_by_id(&pool, created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_by_id_not_found(pool: PgPool) {
        let found = get_by_id(&pool, 99999).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update(pool: PgPool) {
        let created = create(&pool, &create_req("2025-01-15")).await.unwrap();
        let req = UpdateBulkTankTest {
            date: None,
            fat: Some(4.1),
            protein: Some(3.5),
            lactose: None,
            scc: None,
            ffa: None,
        };
        let updated = update(&pool, created.id, &req).await.unwrap();
        assert_eq!(updated.fat, 4.1);
        assert_eq!(updated.protein, 3.5);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete(pool: PgPool) {
        let created = create(&pool, &create_req("2025-01-15")).await.unwrap();
        delete(&pool, created.id).await.unwrap();
        let found = get_by_id(&pool, created.id).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_not_found(pool: PgPool) {
        let result = delete(&pool, 99999).await;
        assert!(result.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_filter_by_date(pool: PgPool) {
        create(&pool, &create_req("2025-01-10")).await.unwrap();
        create(&pool, &create_req("2025-01-20")).await.unwrap();
        let filter = BulkTankFilter {
            from_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
            till_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 25).unwrap()),
            page: None,
            per_page: None,
        };
        let tanks = list(&pool, &filter).await.unwrap();
        assert_eq!(tanks.len(), 1);
    }
}
