use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::reproduction::*;
use crate::services::animal_service;

pub async fn list_calvings(
    pool: &PgPool,
    filter: &ReproductionFilter,
) -> Result<Vec<Calving>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, Calving>(
        "SELECT * FROM calvings WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR calving_date >= $2) AND ($3::date IS NULL OR calving_date <= $3)
         ORDER BY calving_date DESC LIMIT $4 OFFSET $5",
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

pub async fn count_calvings(pool: &PgPool, filter: &ReproductionFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM calvings WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR calving_date >= $2) AND ($3::date IS NULL OR calving_date <= $3)",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn get_calving(pool: &PgPool, id: i32) -> Result<Option<Calving>, AppError> {
    sqlx::query_as::<_, Calving>("SELECT * FROM calvings WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create_calving(pool: &PgPool, req: &CreateCalving) -> Result<Calving, AppError> {
    animal_service::ensure_exists(pool, req.animal_id).await?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    let calving = sqlx::query_as::<_, Calving>(
        "INSERT INTO calvings (animal_id, calving_date, remarks, lac_number)
         VALUES ($1,$2,$3,$4) RETURNING *",
    )
    .bind(req.animal_id)
    .bind(req.calving_date)
    .bind(&req.remarks)
    .bind(req.lac_number)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::Database)?;

    if let Some(calves) = &req.calves {
        for calf in calves {
            sqlx::query(
                "INSERT INTO calves (calving_id, life_number, gender, birth_remark, keep, weight,
                 born_dead, animal_number, calf_name, hair_color_code, born_dead_reason_id)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
            )
            .bind(calving.id)
            .bind(&calf.life_number)
            .bind(&calf.gender)
            .bind(&calf.birth_remark)
            .bind(calf.keep)
            .bind(calf.weight)
            .bind(calf.born_dead)
            .bind(calf.animal_number)
            .bind(&calf.calf_name)
            .bind(&calf.hair_color_code)
            .bind(calf.born_dead_reason_id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::Database)?;
        }
    }

    tx.commit().await.map_err(AppError::Database)?;
    Ok(calving)
}

pub async fn list_inseminations(
    pool: &PgPool,
    filter: &ReproductionFilter,
) -> Result<Vec<Insemination>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, Insemination>(
        "SELECT * FROM inseminations WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR insemination_date >= $2) AND ($3::date IS NULL OR insemination_date <= $3)
         ORDER BY insemination_date DESC LIMIT $4 OFFSET $5",
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

pub async fn count_inseminations(
    pool: &PgPool,
    filter: &ReproductionFilter,
) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM inseminations WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR insemination_date >= $2) AND ($3::date IS NULL OR insemination_date <= $3)",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create_insemination(
    pool: &PgPool,
    req: &CreateInsemination,
) -> Result<Insemination, AppError> {
    animal_service::ensure_exists(pool, req.animal_id).await?;

    sqlx::query_as::<_, Insemination>(
        "INSERT INTO inseminations (animal_id, insemination_date, sire_code, insemination_type, charge_number)
         VALUES ($1,$2,$3,$4,$5) RETURNING *",
    )
    .bind(req.animal_id)
    .bind(req.insemination_date)
    .bind(&req.sire_code)
    .bind(&req.insemination_type)
    .bind(&req.charge_number)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn list_pregnancies(
    pool: &PgPool,
    filter: &ReproductionFilter,
) -> Result<Vec<Pregnancy>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, Pregnancy>(
        "SELECT * FROM pregnancies WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR pregnancy_date >= $2) AND ($3::date IS NULL OR pregnancy_date <= $3)
         ORDER BY pregnancy_date DESC LIMIT $4 OFFSET $5",
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

pub async fn count_pregnancies(
    pool: &PgPool,
    filter: &ReproductionFilter,
) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pregnancies WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR pregnancy_date >= $2) AND ($3::date IS NULL OR pregnancy_date <= $3)",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create_pregnancy(pool: &PgPool, req: &CreatePregnancy) -> Result<Pregnancy, AppError> {
    animal_service::ensure_exists(pool, req.animal_id).await?;

    sqlx::query_as::<_, Pregnancy>(
        "INSERT INTO pregnancies (animal_id, pregnancy_date, pregnancy_type, insemination_date)
         VALUES ($1,$2,$3,$4) RETURNING *",
    )
    .bind(req.animal_id)
    .bind(req.pregnancy_date)
    .bind(&req.pregnancy_type)
    .bind(req.insemination_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn list_heats(pool: &PgPool, filter: &ReproductionFilter) -> Result<Vec<Heat>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, Heat>(
        "SELECT * FROM heats WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR heat_date >= $2) AND ($3::date IS NULL OR heat_date <= $3)
         ORDER BY heat_date DESC LIMIT $4 OFFSET $5",
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

pub async fn count_heats(pool: &PgPool, filter: &ReproductionFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM heats WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR heat_date >= $2) AND ($3::date IS NULL OR heat_date <= $3)",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create_heat(pool: &PgPool, req: &CreateHeat) -> Result<Heat, AppError> {
    animal_service::ensure_exists(pool, req.animal_id).await?;

    sqlx::query_as::<_, Heat>("INSERT INTO heats (animal_id, heat_date) VALUES ($1,$2) RETURNING *")
        .bind(req.animal_id)
        .bind(req.heat_date)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn list_dryoffs(
    pool: &PgPool,
    filter: &ReproductionFilter,
) -> Result<Vec<DryOff>, AppError> {
    let pag = crate::models::pagination::Pagination::from_filter(filter.page, filter.per_page);

    sqlx::query_as::<_, DryOff>(
        "SELECT * FROM dry_offs WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR dry_off_date >= $2) AND ($3::date IS NULL OR dry_off_date <= $3)
         ORDER BY dry_off_date DESC LIMIT $4 OFFSET $5",
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

pub async fn count_dryoffs(pool: &PgPool, filter: &ReproductionFilter) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM dry_offs WHERE ($1::text IS NULL OR animal_id::text LIKE $1 || '%')
         AND ($2::date IS NULL OR dry_off_date >= $2) AND ($3::date IS NULL OR dry_off_date <= $3)",
    )
    .bind(filter.animal_id.clone())
    .bind(filter.from_date)
    .bind(filter.till_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(row.0)
}

pub async fn create_dryoff(pool: &PgPool, req: &CreateDryOff) -> Result<DryOff, AppError> {
    animal_service::ensure_exists(pool, req.animal_id).await?;

    sqlx::query_as::<_, DryOff>(
        "INSERT INTO dry_offs (animal_id, dry_off_date) VALUES ($1,$2) RETURNING *",
    )
    .bind(req.animal_id)
    .bind(req.dry_off_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_calving(
    pool: &PgPool,
    id: i32,
    req: &UpdateCalving,
) -> Result<Calving, AppError> {
    sqlx::query_as::<_, Calving>(
        "UPDATE calvings SET
         calving_date = COALESCE($2, calving_date),
         remarks = COALESCE($3, remarks),
         lac_number = COALESCE($4, lac_number)
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(req.calving_date)
    .bind(&req.remarks)
    .bind(req.lac_number)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_calving(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM calvings WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Calving {} not found", id)));
    }
    Ok(())
}

pub async fn get_insemination(pool: &PgPool, id: i32) -> Result<Option<Insemination>, AppError> {
    sqlx::query_as::<_, Insemination>("SELECT * FROM inseminations WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn update_insemination(
    pool: &PgPool,
    id: i32,
    req: &UpdateInsemination,
) -> Result<Insemination, AppError> {
    sqlx::query_as::<_, Insemination>(
        "UPDATE inseminations SET
         insemination_date = COALESCE($2, insemination_date),
         sire_code = COALESCE($3, sire_code),
         insemination_type = COALESCE($4, insemination_type),
         charge_number = COALESCE($5, charge_number)
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(req.insemination_date)
    .bind(&req.sire_code)
    .bind(&req.insemination_type)
    .bind(&req.charge_number)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_insemination(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM inseminations WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Insemination {} not found", id)));
    }
    Ok(())
}

pub async fn get_pregnancy(pool: &PgPool, id: i32) -> Result<Option<Pregnancy>, AppError> {
    sqlx::query_as::<_, Pregnancy>("SELECT * FROM pregnancies WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn update_pregnancy(
    pool: &PgPool,
    id: i32,
    req: &UpdatePregnancy,
) -> Result<Pregnancy, AppError> {
    sqlx::query_as::<_, Pregnancy>(
        "UPDATE pregnancies SET
         pregnancy_date = COALESCE($2, pregnancy_date),
         pregnancy_type = COALESCE($3, pregnancy_type),
         insemination_date = COALESCE($4, insemination_date)
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(req.pregnancy_date)
    .bind(&req.pregnancy_type)
    .bind(req.insemination_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_pregnancy(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM pregnancies WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Pregnancy {} not found", id)));
    }
    Ok(())
}

pub async fn get_heat(pool: &PgPool, id: i32) -> Result<Option<Heat>, AppError> {
    sqlx::query_as::<_, Heat>("SELECT * FROM heats WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn update_heat(pool: &PgPool, id: i32, req: &UpdateHeat) -> Result<Heat, AppError> {
    sqlx::query_as::<_, Heat>(
        "UPDATE heats SET heat_date = COALESCE($2, heat_date) WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(req.heat_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_heat(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM heats WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Heat {} not found", id)));
    }
    Ok(())
}

pub async fn get_dryoff(pool: &PgPool, id: i32) -> Result<Option<DryOff>, AppError> {
    sqlx::query_as::<_, DryOff>("SELECT * FROM dry_offs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn update_dryoff(pool: &PgPool, id: i32, req: &UpdateDryOff) -> Result<DryOff, AppError> {
    sqlx::query_as::<_, DryOff>(
        "UPDATE dry_offs SET dry_off_date = COALESCE($2, dry_off_date) WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(req.dry_off_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_dryoff(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM dry_offs WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("DryOff {} not found", id)));
    }
    Ok(())
}

pub async fn current_status(pool: &PgPool) -> Result<Vec<serde_json::Value>, AppError> {
    sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT json_build_object(
            'animal_id', a.id,
            'life_number', a.life_number,
            'name', a.name,
            'production_status', CASE
                WHEN d.animal_id IS NOT NULL THEN 'dry_off'
                WHEN c.animal_id IS NOT NULL THEN 'in_lactation'
                ELSE 'young_stock'
            END,
            'last_calving_date', cal.max_calving,
            'last_insemination_date', ins.max_insem,
            'last_heat_date', ht.max_heat
        )
        FROM animals a
        LEFT JOIN LATERAL (SELECT animal_id FROM dry_offs WHERE animal_id = a.id AND dry_off_date <= CURRENT_DATE LIMIT 1) d ON true
        LEFT JOIN LATERAL (SELECT animal_id FROM calvings WHERE animal_id = a.id AND calving_date <= CURRENT_DATE LIMIT 1) c ON true
        LEFT JOIN LATERAL (SELECT MAX(calving_date) as max_calving FROM calvings WHERE animal_id = a.id) cal ON true
        LEFT JOIN LATERAL (SELECT MAX(insemination_date) as max_insem FROM inseminations WHERE animal_id = a.id) ins ON true
        LEFT JOIN LATERAL (SELECT MAX(heat_date) as max_heat FROM heats WHERE animal_id = a.id) ht ON true
        WHERE a.active = true AND a.gender = 'female'"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
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
    async fn test_create_calving(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req = CreateCalving {
            animal_id,
            calving_date: chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            remarks: Some("Normal".into()),
            lac_number: Some(1),
            calves: None,
        };
        let calving = create_calving(&pool, &req).await.unwrap();
        assert_eq!(calving.animal_id, animal_id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_get_calving(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req = CreateCalving {
            animal_id,
            calving_date: chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            remarks: None,
            lac_number: None,
            calves: None,
        };
        let created = create_calving(&pool, &req).await.unwrap();
        let found = get_calving(&pool, created.id).await.unwrap();
        assert!(found.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_calving_with_calves(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req = CreateCalving {
            animal_id,
            calving_date: chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            remarks: None,
            lac_number: None,
            calves: Some(vec![CreateCalf {
                life_number: Some("CL001".into()),
                gender: GenderType::Female,
                birth_remark: Some(BirthRemarkType::Normal),
                keep: Some(true),
                weight: Some(35.0),
                born_dead: Some(false),
                animal_number: None,
                calf_name: Some("Calf1".into()),
                hair_color_code: None,
                born_dead_reason_id: None,
            }]),
        };
        let calving = create_calving(&pool, &req).await.unwrap();
        assert_eq!(calving.animal_id, animal_id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_calvings_empty(pool: PgPool) {
        let filter = ReproductionFilter {
            animal_id: None,
            life_number: None,
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let calvings = list_calvings(&pool, &filter).await.unwrap();
        assert!(calvings.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_insemination(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req = CreateInsemination {
            animal_id,
            insemination_date: chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            sire_code: Some("SIRE001".into()),
            insemination_type: Some("AI".into()),
            charge_number: None,
        };
        let ins = create_insemination(&pool, &req).await.unwrap();
        assert_eq!(ins.animal_id, animal_id);
        assert_eq!(ins.sire_code.as_deref(), Some("SIRE001"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_inseminations_filter(pool: PgPool) {
        let a1 = seed_cow(&pool).await;
        let a2 = seed_cow(&pool).await;
        create_insemination(
            &pool,
            &CreateInsemination {
                animal_id: a1,
                insemination_date: chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
                sire_code: None,
                insemination_type: None,
                charge_number: None,
            },
        )
        .await
        .unwrap();
        create_insemination(
            &pool,
            &CreateInsemination {
                animal_id: a2,
                insemination_date: chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
                sire_code: None,
                insemination_type: None,
                charge_number: None,
            },
        )
        .await
        .unwrap();
        let filter = ReproductionFilter {
            animal_id: Some(a1.to_string()),
            life_number: None,
            from_date: None,
            till_date: None,
            page: None,
            per_page: None,
        };
        let list = list_inseminations(&pool, &filter).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_pregnancy(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req = CreatePregnancy {
            animal_id,
            pregnancy_date: chrono::NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            pregnancy_type: Some("ultrasound".into()),
            insemination_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()),
        };
        let preg = create_pregnancy(&pool, &req).await.unwrap();
        assert_eq!(preg.animal_id, animal_id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_heat(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req = CreateHeat {
            animal_id,
            heat_date: chrono::NaiveDate::from_ymd_opt(2024, 5, 1).unwrap(),
        };
        let heat = create_heat(&pool, &req).await.unwrap();
        assert_eq!(heat.animal_id, animal_id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_list_heats_filter_by_date(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        create_heat(
            &pool,
            &CreateHeat {
                animal_id,
                heat_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            },
        )
        .await
        .unwrap();
        create_heat(
            &pool,
            &CreateHeat {
                animal_id,
                heat_date: chrono::NaiveDate::from_ymd_opt(2024, 5, 10).unwrap(),
            },
        )
        .await
        .unwrap();
        let filter = ReproductionFilter {
            animal_id: None,
            life_number: None,
            from_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 5, 1).unwrap()),
            till_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 5, 31).unwrap()),
            page: None,
            per_page: None,
        };
        let heats = list_heats(&pool, &filter).await.unwrap();
        assert_eq!(heats.len(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_create_dryoff(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let req = CreateDryOff {
            animal_id,
            dry_off_date: chrono::NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
        };
        let dry = create_dryoff(&pool, &req).await.unwrap();
        assert_eq!(dry.animal_id, animal_id);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_current_status(pool: PgPool) {
        let animal_id = seed_cow(&pool).await;
        let status = current_status(&pool).await.unwrap();
        assert_eq!(status.len(), 1);
        let obj = &status[0];
        assert_eq!(obj["animal_id"], animal_id);
    }
}
