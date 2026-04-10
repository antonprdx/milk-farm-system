use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::animal::{Animal, AnimalFilter, CreateAnimal, UpdateAnimal};

pub async fn ensure_exists(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM animals WHERE id = $1 AND active = true)",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    if !exists {
        return Err(AppError::NotFound(format!(
            "Животное с ID {} не найдено или неактивно",
            id
        )));
    }
    Ok(())
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

pub async fn list(pool: &PgPool, filter: &AnimalFilter) -> Result<Vec<Animal>, AppError> {
    let pag = crate::models::pagination::Pagination::new(filter.page, filter.per_page, 20, 100);
    let name_pattern = filter.name.as_ref().map(|n| format!("%{}%", escape_like(n)));

    sqlx::query_as::<_, Animal>(
        "SELECT * FROM animals WHERE ($1::text IS NULL OR life_number = $1)
         AND ($2::text IS NULL OR ucn_number = $2)
         AND ($3::bool IS NULL OR active = $3)
         AND ($4::gender_type IS NULL OR gender = $4)
         AND ($5::text IS NULL OR name ILIKE $5)
         ORDER BY id LIMIT $6 OFFSET $7",
    )
    .bind(&filter.life_number)
    .bind(&filter.ucn_number)
    .bind(filter.active)
    .bind(&filter.gender)
    .bind(&name_pattern)
    .bind(pag.per_page)
    .bind(pag.offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count(pool: &PgPool, filter: &AnimalFilter) -> Result<i64, AppError> {
    let name_pattern = filter.name.as_ref().map(|n| format!("%{}%", escape_like(n)));

    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM animals WHERE ($1::text IS NULL OR life_number = $1)
         AND ($2::text IS NULL OR ucn_number = $2)
         AND ($3::bool IS NULL OR active = $3)
         AND ($4::gender_type IS NULL OR gender = $4)
         AND ($5::text IS NULL OR name ILIKE $5)",
    )
    .bind(&filter.life_number)
    .bind(&filter.ucn_number)
    .bind(filter.active)
    .bind(&filter.gender)
    .bind(&name_pattern)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(row.0)
}

pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Animal>, AppError> {
    sqlx::query_as::<_, Animal>("SELECT * FROM animals WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create(pool: &PgPool, req: &CreateAnimal) -> Result<Animal, AppError> {
    sqlx::query_as::<_, Animal>(
        "INSERT INTO animals (life_number, name, user_number, gender, birth_date,
         hair_color_code, father_life_number, mother_life_number, description,
         ucn_number, use_as_sire, location, group_number, keep, gestation, responder_number)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16) RETURNING *",
    )
    .bind(&req.life_number)
    .bind(&req.name)
    .bind(req.user_number)
    .bind(&req.gender)
    .bind(req.birth_date)
    .bind(&req.hair_color_code)
    .bind(&req.father_life_number)
    .bind(&req.mother_life_number)
    .bind(&req.description)
    .bind(&req.ucn_number)
    .bind(req.use_as_sire)
    .bind(&req.location)
    .bind(req.group_number)
    .bind(req.keep)
    .bind(req.gestation)
    .bind(&req.responder_number)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update(pool: &PgPool, id: i32, req: &UpdateAnimal) -> Result<Animal, AppError> {
    sqlx::query_as::<_, Animal>(
        "UPDATE animals SET name = COALESCE($2, name),
         hair_color_code = COALESCE($3, hair_color_code),
         description = COALESCE($4, description),
         ucn_number = COALESCE($5, ucn_number),
         use_as_sire = COALESCE($6, use_as_sire),
         location = COALESCE($7, location),
         group_number = COALESCE($8, group_number),
         keep = COALESCE($9, keep),
         gestation = COALESCE($10, gestation),
         responder_number = COALESCE($11, responder_number),
         active = COALESCE($12, active),
         updated_at = NOW()
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.hair_color_code)
    .bind(&req.description)
    .bind(&req.ucn_number)
    .bind(req.use_as_sire)
    .bind(&req.location)
    .bind(req.group_number)
    .bind(req.keep)
    .bind(req.gestation)
    .bind(&req.responder_number)
    .bind(req.active)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("UPDATE animals SET active = false, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Animal {} not found", id)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::animal::{CreateAnimal, GenderType, UpdateAnimal, AnimalFilter};

    fn create_female_req() -> CreateAnimal {
        CreateAnimal {
            life_number: Some("LN001".into()),
            name: Some("Burenka".into()),
            user_number: Some(42),
            gender: GenderType::Female,
            birth_date: chrono::NaiveDate::from_ymd_opt(2020, 3, 15).unwrap(),
            hair_color_code: None,
            father_life_number: None,
            mother_life_number: None,
            description: Some("Test cow".into()),
            ucn_number: Some("UCN001".into()),
            use_as_sire: None,
            location: None,
            group_number: None,
            keep: None,
            gestation: None,
            responder_number: None,
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_animal(pool: PgPool) {
        let animal = create(&pool, &create_female_req()).await.unwrap();
        assert_eq!(animal.name.as_deref(), Some("Burenka"));
        assert!(animal.active);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_by_id_exists(pool: PgPool) {
        let created = create(&pool, &create_female_req()).await.unwrap();
        let found = get_by_id(&pool, created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_by_id_not_exists(pool: PgPool) {
        let found = get_by_id(&pool, 99999).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_animals(pool: PgPool) {
        create(&pool, &create_female_req()).await.unwrap();
        let filter = AnimalFilter { life_number: None, ucn_number: None, name: None, active: None, gender: None, page: None, per_page: None };
        let animals = list(&pool, &filter).await.unwrap();
        assert_eq!(animals.len(), 1);
        let total = count(&pool, &filter).await.unwrap();
        assert_eq!(total, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_filter_by_active(pool: PgPool) {
        create(&pool, &create_female_req()).await.unwrap();
        let filter = AnimalFilter { life_number: None, ucn_number: None, name: None, active: Some(false), gender: None, page: None, per_page: None };
        let animals = list(&pool, &filter).await.unwrap();
        assert!(animals.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_filter_by_gender(pool: PgPool) {
        create(&pool, &create_female_req()).await.unwrap();
        let filter = AnimalFilter { life_number: None, ucn_number: None, name: None, active: None, gender: Some(GenderType::Male), page: None, per_page: None };
        let animals = list(&pool, &filter).await.unwrap();
        assert!(animals.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_filter_by_ucn(pool: PgPool) {
        create(&pool, &create_female_req()).await.unwrap();
        let filter = AnimalFilter { life_number: None, ucn_number: Some("UCN001".into()), name: None, active: None, gender: None, page: None, per_page: None };
        let animals = list(&pool, &filter).await.unwrap();
        assert_eq!(animals.len(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_animal(pool: PgPool) {
        let created = create(&pool, &create_female_req()).await.unwrap();
        let update_req = UpdateAnimal {
            name: Some("NewName".into()),
            active: Some(false),
            hair_color_code: None,
            description: None,
            ucn_number: None,
            use_as_sire: None,
            location: None,
            group_number: None,
            keep: None,
            gestation: None,
            responder_number: None,
        };
        let updated = update(&pool, created.id, &update_req).await.unwrap();
        assert_eq!(updated.name.as_deref(), Some("NewName"));
        assert!(!updated.active);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_animal(pool: PgPool) {
        let created = create(&pool, &create_female_req()).await.unwrap();
        delete(&pool, created.id).await.unwrap();
        let found = get_by_id(&pool, created.id).await.unwrap();
        assert!(found.is_some());
        assert!(!found.unwrap().active);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_delete_nonexistent(pool: PgPool) {
        let result = delete(&pool, 99999).await;
        assert!(result.is_err());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_pagination(pool: PgPool) {
        for _ in 0..5 {
            create(&pool, &create_female_req()).await.unwrap();
        }
        let filter = AnimalFilter { life_number: None, ucn_number: None, name: None, active: None, gender: None, page: Some(1), per_page: Some(2) };
        let page1 = list(&pool, &filter).await.unwrap();
        assert_eq!(page1.len(), 2);
        let filter2 = AnimalFilter { life_number: None, ucn_number: None, name: None, active: None, gender: None, page: Some(3), per_page: Some(2) };
        let page3 = list(&pool, &filter2).await.unwrap();
        assert_eq!(page3.len(), 1);
    }
}
