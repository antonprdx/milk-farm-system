use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::timeline::TimelineEvent;

pub async fn list(
    pool: &PgPool,
    animal_id: i32,
    page: Option<i64>,
    per_page: Option<i64>,
) -> Result<Vec<TimelineEvent>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(page, per_page);

    sqlx::query_as::<_, TimelineEvent>(
        "SELECT date, event_type, description FROM (
            SELECT calving_date::date as date, 'Отёл'::text as event_type,
                   COALESCE(remarks, 'Отёл') as description
            FROM calvings WHERE animal_id = $1

            UNION ALL

            SELECT insemination_date::date, 'Осеменение',
                   CONCAT(COALESCE(sire_code, ''), ' ', COALESCE(insemination_type, ''))
            FROM inseminations WHERE animal_id = $1

            UNION ALL

            SELECT pregnancy_date::date, 'Диагностика стельности',
                   COALESCE(pregnancy_type, 'Проверка')
            FROM pregnancies WHERE animal_id = $1

            UNION ALL

            SELECT heat_date::date, 'Охота', 'Зафиксирована охота'
            FROM heats WHERE animal_id = $1

            UNION ALL

            SELECT dry_off_date::date, 'Запуск', 'Животное запущено'
            FROM dry_offs WHERE animal_id = $1

            UNION ALL

            SELECT date::date, 'Надой', CONCAT(milk_amount::text, ' кг')
            FROM milk_day_productions WHERE animal_id = $1
        ) events
        ORDER BY date DESC
        LIMIT $2 OFFSET $3",
    )
    .bind(animal_id)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count(pool: &PgPool, animal_id: i32) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM (
            SELECT calving_date::date FROM calvings WHERE animal_id = $1
            UNION ALL
            SELECT insemination_date::date FROM inseminations WHERE animal_id = $1
            UNION ALL
            SELECT pregnancy_date::date FROM pregnancies WHERE animal_id = $1
            UNION ALL
            SELECT heat_date::date FROM heats WHERE animal_id = $1
            UNION ALL
            SELECT dry_off_date::date FROM dry_offs WHERE animal_id = $1
            UNION ALL
            SELECT date::date FROM milk_day_productions WHERE animal_id = $1
        ) events",
    )
    .bind(animal_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}
