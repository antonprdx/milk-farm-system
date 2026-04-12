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

pub async fn create_day_amount(
    pool: &PgPool,
    data: &CreateFeedDayAmount,
) -> Result<FeedDayAmount, AppError> {
    sqlx::query_as::<_, FeedDayAmount>(
        "INSERT INTO feed_day_amounts (animal_id, feed_date, feed_number, total, rest_feed)
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(data.animal_id)
    .bind(data.feed_date)
    .bind(data.feed_number)
    .bind(data.total)
    .bind(data.rest_feed)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn create_feed_type(pool: &PgPool, data: &CreateFeedType) -> Result<FeedType, AppError> {
    sqlx::query_as::<_, FeedType>(
        "INSERT INTO feed_types (number_of_feed_type, feed_type, name, description, dry_matter_percentage, stock_attention_level, price)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
    )
    .bind(data.number_of_feed_type)
    .bind(&data.feed_type)
    .bind(&data.name)
    .bind(&data.description)
    .bind(data.dry_matter_percentage)
    .bind(data.stock_attention_level)
    .bind(data.price)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_feed_type(
    pool: &PgPool,
    id: i32,
    data: &UpdateFeedType,
) -> Result<FeedType, AppError> {
    let existing = sqlx::query_as::<_, FeedType>("SELECT * FROM feed_types WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Тип корма не найден".into()))?;

    sqlx::query_as::<_, FeedType>(
        "UPDATE feed_types SET number_of_feed_type = $1, feed_type = $2, name = $3,
         description = $4, dry_matter_percentage = $5, stock_attention_level = $6, price = $7
         WHERE id = $8 RETURNING *",
    )
    .bind(
        data.number_of_feed_type
            .unwrap_or(existing.number_of_feed_type),
    )
    .bind(data.feed_type.as_deref().unwrap_or(&existing.feed_type))
    .bind(data.name.as_deref().unwrap_or(&existing.name))
    .bind(
        data.description
            .as_deref()
            .or(existing.description.as_deref()),
    )
    .bind(
        data.dry_matter_percentage
            .unwrap_or(existing.dry_matter_percentage),
    )
    .bind(
        data.stock_attention_level
            .or(existing.stock_attention_level),
    )
    .bind(data.price.unwrap_or(existing.price))
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_feed_type(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM feed_types WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Тип корма не найден".into()));
    }
    Ok(())
}

pub async fn create_feed_group(
    pool: &PgPool,
    data: &CreateFeedGroup,
) -> Result<FeedGroup, AppError> {
    sqlx::query_as::<_, FeedGroup>(
        "INSERT INTO feed_groups (name, min_milk_yield, max_milk_yield, avg_milk_yield,
         avg_milk_fat, avg_milk_protein, avg_weight, max_robot_feed_types,
         max_feed_intake_robot, min_feed_intake_robot, number_of_cows)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *",
    )
    .bind(&data.name)
    .bind(data.min_milk_yield)
    .bind(data.max_milk_yield)
    .bind(data.avg_milk_yield)
    .bind(data.avg_milk_fat)
    .bind(data.avg_milk_protein)
    .bind(data.avg_weight)
    .bind(data.max_robot_feed_types)
    .bind(data.max_feed_intake_robot)
    .bind(data.min_feed_intake_robot)
    .bind(data.number_of_cows)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_feed_group(
    pool: &PgPool,
    id: i32,
    data: &UpdateFeedGroup,
) -> Result<FeedGroup, AppError> {
    let existing = sqlx::query_as::<_, FeedGroup>("SELECT * FROM feed_groups WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Группа корма не найдена".into()))?;

    sqlx::query_as::<_, FeedGroup>(
        "UPDATE feed_groups SET name = $1, min_milk_yield = $2, max_milk_yield = $3,
         avg_milk_yield = $4, avg_milk_fat = $5, avg_milk_protein = $6, avg_weight = $7,
         max_robot_feed_types = $8, max_feed_intake_robot = $9, min_feed_intake_robot = $10,
         number_of_cows = $11 WHERE id = $12 RETURNING *",
    )
    .bind(data.name.as_deref().unwrap_or(&existing.name))
    .bind(data.min_milk_yield.or(existing.min_milk_yield))
    .bind(data.max_milk_yield.or(existing.max_milk_yield))
    .bind(data.avg_milk_yield.or(existing.avg_milk_yield))
    .bind(data.avg_milk_fat.or(existing.avg_milk_fat))
    .bind(data.avg_milk_protein.or(existing.avg_milk_protein))
    .bind(data.avg_weight.or(existing.avg_weight))
    .bind(data.max_robot_feed_types.or(existing.max_robot_feed_types))
    .bind(
        data.max_feed_intake_robot
            .or(existing.max_feed_intake_robot),
    )
    .bind(
        data.min_feed_intake_robot
            .or(existing.min_feed_intake_robot),
    )
    .bind(data.number_of_cows.or(existing.number_of_cows))
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_feed_group(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM feed_groups WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Группа корма не найдена".into()));
    }
    Ok(())
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
