use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;

use crate::config::LelyConfig;
use crate::lely::crypto;
use crate::lely::mapper::AnimalCache;
use crate::lely::models::*;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LelyConfigRow {
    pub enabled: bool,
    pub base_url: String,
    pub username: String,
    pub password_encrypted: String,
    pub farm_key_encrypted: String,
    pub sync_interval_secs: i64,
}

pub async fn load_config(pool: &PgPool, encryption_key: &str) -> Result<LelyConfig, anyhow::Error> {
    let row = sqlx::query_as::<_, LelyConfigRow>(
        "SELECT enabled, base_url, username, password_encrypted, farm_key_encrypted, sync_interval_secs \
         FROM lely_config WHERE id = 1",
    )
    .fetch_one(pool)
    .await?;

    let password = if row.password_encrypted.is_empty() {
        String::new()
    } else {
        crypto::decrypt(&row.password_encrypted, encryption_key).unwrap_or_default()
    };

    let farm_key = if row.farm_key_encrypted.is_empty() {
        String::new()
    } else {
        crypto::decrypt(&row.farm_key_encrypted, encryption_key).unwrap_or_default()
    };

    Ok(LelyConfig {
        enabled: row.enabled,
        base_url: row.base_url,
        username: row.username,
        password,
        farm_key,
        sync_interval_secs: row.sync_interval_secs as u64,
    })
}

pub async fn save_config(
    pool: &PgPool,
    cfg: &LelyConfig,
    encryption_key: &str,
) -> Result<(), anyhow::Error> {
    let password_encrypted = if cfg.password.is_empty() {
        String::new()
    } else {
        crypto::encrypt(&cfg.password, encryption_key)?
    };
    let farm_key_encrypted = if cfg.farm_key.is_empty() {
        String::new()
    } else {
        crypto::encrypt(&cfg.farm_key, encryption_key)?
    };

    sqlx::query(
        "UPDATE lely_config SET enabled = $1, base_url = $2, username = $3, \
         password_encrypted = $4, farm_key_encrypted = $5, sync_interval_secs = $6, updated_at = NOW() \
         WHERE id = 1",
    )
    .bind(cfg.enabled)
    .bind(&cfg.base_url)
    .bind(&cfg.username)
    .bind(&password_encrypted)
    .bind(&farm_key_encrypted)
    .bind(cfg.sync_interval_secs as i64)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_config_masked(pool: &PgPool) -> Result<LelyConfigRow, sqlx::Error> {
    sqlx::query_as::<_, LelyConfigRow>(
        "SELECT enabled, base_url, username, \
         password_encrypted, farm_key_encrypted, sync_interval_secs \
         FROM lely_config WHERE id = 1",
    )
    .fetch_one(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SyncState {
    pub id: i32,
    pub entity_type: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub status: String,
    pub records_synced: i64,
    pub error_message: Option<String>,
    pub updated_at: DateTime<Utc>,
}

pub async fn get_sync_state(
    pool: &PgPool,
    entity_type: &str,
) -> Result<Option<SyncState>, sqlx::Error> {
    sqlx::query_as::<_, SyncState>(
        "SELECT id, entity_type, last_synced_at, status, records_synced, error_message, updated_at \
         FROM lely_sync_state WHERE entity_type = $1",
    )
    .bind(entity_type)
    .fetch_optional(pool)
    .await
}

pub async fn get_all_sync_states(pool: &PgPool) -> Result<Vec<SyncState>, sqlx::Error> {
    sqlx::query_as::<_, SyncState>(
        "SELECT id, entity_type, last_synced_at, status, records_synced, error_message, updated_at \
         FROM lely_sync_state ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

pub async fn update_sync_state(
    pool: &PgPool,
    entity_type: &str,
    status: &str,
    records: i64,
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE lely_sync_state \
         SET last_synced_at = NOW(), status = $2, records_synced = $3, error_message = $4, updated_at = NOW() \
         WHERE entity_type = $1",
    )
    .bind(entity_type)
    .bind(status)
    .bind(records)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn try_acquire_lock(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let row: (bool,) = sqlx::query_as("SELECT pg_try_advisory_lock(20260411)")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn release_lock(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_unlock(20260411)")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn upsert_animals(
    pool: &PgPool,
    records: &[AnimalResponse],
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(ln) = &r.life_number else { continue };
        let birth = parse_lely_date(&r.birth_date);
        let Some(birth_date) = birth else {
            tracing::warn!(life_number = %ln, "Животное без даты рождения, пропущено");
            continue;
        };
        let gender = match r.gender.as_deref() {
            Some("Male") => "male",
            Some("Female") => "female",
            _ => "female",
        };

        let result = sqlx::query(
            "INSERT INTO animals (life_number, name, user_number, gender, birth_date, hair_color_code, \
             father_life_number, mother_life_number, description, ucn_number, use_as_sire, location, \
             group_number, keep, gestation, responder_number, active, updated_at) \
             VALUES ($1, $2, $3, $4::gender_type, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, true, NOW()) \
             ON CONFLICT DO NOTHING"
        )
        .bind(ln)
        .bind(&r.name)
        .bind(r.user_number)
        .bind(gender)
        .bind(birth_date)
        .bind(&r.hair_color_code)
        .bind(&r.father_life_number)
        .bind(&r.mother_life_number)
        .bind(&r.description)
        .bind(&r.ucn_number)
        .bind(r.use_as_sire)
        .bind(&r.location)
        .bind(r.group_number)
        .bind(r.keep)
        .bind(r.gestation)
        .bind(&r.responder_number)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_milk_day_productions(
    pool: &PgPool,
    records: &[DayProduction],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "milk_day_productions") else {
            continue;
        };
        let Some(date) = parse_lely_date(&r.date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO milk_day_productions (animal_id, date, milk_amount, avg_amount, avg_weight, isk) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (animal_id, date) DO UPDATE SET \
             milk_amount = COALESCE(EXCLUDED.milk_amount, milk_day_productions.milk_amount), \
             avg_amount = COALESCE(EXCLUDED.avg_amount, milk_day_productions.avg_amount), \
             avg_weight = COALESCE(EXCLUDED.avg_weight, milk_day_productions.avg_weight), \
             isk = COALESCE(EXCLUDED.isk, milk_day_productions.isk)"
        )
        .bind(animal_id)
        .bind(date)
        .bind(r.milk_day_production)
        .bind(r.milk_day_production_average)
        .bind(r.average_weight)
        .bind(r.isk)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_milk_visits(
    pool: &PgPool,
    records: &[MilkVisit],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "milk_visits") else {
            continue;
        };
        let Some(dt) = parse_lely_datetime(&r.milking_date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO milk_visits (animal_id, visit_datetime, milk_amount, duration_seconds, milk_destination) \
             VALUES ($1, $2, $3, NULL, $4) \
             ON CONFLICT (animal_id, visit_datetime) DO UPDATE SET \
             milk_amount = COALESCE(EXCLUDED.milk_amount, milk_visits.milk_amount), \
             milk_destination = COALESCE(EXCLUDED.milk_destination, milk_visits.milk_destination)"
        )
        .bind(animal_id)
        .bind(dt)
        .bind(r.milk_yield)
        .bind(r.bottle_number)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_milk_visit_quality(
    pool: &PgPool,
    records: &[MilkVisitQuality],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "milk_visit_quality") else {
            continue;
        };
        let Some(dt) = parse_lely_datetime(&r.milking_date) else {
            continue;
        };
        let start_dt = parse_lely_datetime(&r.milking_start_date);

        let result = sqlx::query(
            "INSERT INTO milk_visit_quality (animal_id, visit_datetime, milking_start_date, device_address, \
             success_milking, milk_yield, bottle_number, milk_temperature, weight, milk_destination, \
             lf_colour_code, lr_colour_code, rf_colour_code, rr_colour_code, \
             lf_conductivity, lr_conductivity, rf_conductivity, rr_conductivity) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18) \
             ON CONFLICT (animal_id, visit_datetime) DO UPDATE SET \
             milk_temperature = COALESCE(EXCLUDED.milk_temperature, milk_visit_quality.milk_temperature), \
             lf_conductivity = COALESCE(EXCLUDED.lf_conductivity, milk_visit_quality.lf_conductivity), \
             lr_conductivity = COALESCE(EXCLUDED.lr_conductivity, milk_visit_quality.lr_conductivity), \
             rf_conductivity = COALESCE(EXCLUDED.rf_conductivity, milk_visit_quality.rf_conductivity), \
             rr_conductivity = COALESCE(EXCLUDED.rr_conductivity, milk_visit_quality.rr_conductivity)"
        )
        .bind(animal_id)
        .bind(dt)
        .bind(start_dt)
        .bind(r.device_address)
        .bind(r.success_milking)
        .bind(r.milk_yield)
        .bind(r.bottle_number)
        .bind(r.milk_temperature)
        .bind(r.weight)
        .bind(r.milk_destination)
        .bind(&r.lf_colour_code)
        .bind(&r.lr_colour_code)
        .bind(&r.rf_colour_code)
        .bind(&r.rr_colour_code)
        .bind(r.lf_conductivity)
        .bind(r.lr_conductivity)
        .bind(r.rf_conductivity)
        .bind(r.rr_conductivity)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_milk_day_quality(
    pool: &PgPool,
    records: &[DayProductionQuality],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "milk_day_quality") else {
            continue;
        };
        let Some(date) = parse_lely_date(&r.date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO milk_quality (animal_id, date, milk_amount, avg_amount, avg_weight, isk, \
             fat_percentage, protein_percentage, lactose_percentage, scc, milkings, refusals, failures) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             ON CONFLICT (animal_id, date) DO UPDATE SET \
             fat_percentage = COALESCE(EXCLUDED.fat_percentage, milk_quality.fat_percentage), \
             protein_percentage = COALESCE(EXCLUDED.protein_percentage, milk_quality.protein_percentage), \
             lactose_percentage = COALESCE(EXCLUDED.lactose_percentage, milk_quality.lactose_percentage), \
             scc = COALESCE(EXCLUDED.scc, milk_quality.scc), \
             milkings = COALESCE(EXCLUDED.milkings, milk_quality.milkings), \
             refusals = COALESCE(EXCLUDED.refusals, milk_quality.refusals), \
             failures = COALESCE(EXCLUDED.failures, milk_quality.failures)"
        )
        .bind(animal_id)
        .bind(date)
        .bind(r.milk_day_production)
        .bind(r.milk_day_production_average)
        .bind(r.average_weight)
        .bind(r.isk)
        .bind(r.fat_percentage)
        .bind(r.protein_percentage)
        .bind(r.lactose_percentage)
        .bind(r.scc)
        .bind(r.mdp_milkings)
        .bind(r.mdp_refusals)
        .bind(r.mdp_failures)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_robot_data(
    pool: &PgPool,
    records: &[RobotData],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "robot_milk_data") else {
            continue;
        };
        let Some(dt) = parse_lely_datetime(&r.milking_date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO robot_milk_data (animal_id, milking_date, device_address, milk_speed, milk_speed_max, \
             lf_milk_time, lr_milk_time, rf_milk_time, rr_milk_time, \
             lf_dead_milk_time, lr_dead_milk_time, rf_dead_milk_time, rr_dead_milk_time, \
             lf_x_position, lf_y_position, lf_z_position, \
             lr_x_position, lr_y_position, lr_z_position, \
             rf_x_position, rf_y_position, rf_z_position, \
             rr_x_position, rr_y_position, rr_z_position) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25) \
             ON CONFLICT (animal_id, milking_date) DO UPDATE SET \
             milk_speed = COALESCE(EXCLUDED.milk_speed, robot_milk_data.milk_speed), \
             milk_speed_max = COALESCE(EXCLUDED.milk_speed_max, robot_milk_data.milk_speed_max)"
        )
        .bind(animal_id)
        .bind(dt)
        .bind(r.device_address)
        .bind(r.milk_speed)
        .bind(r.milk_speed_max)
        .bind(r.lf_milk_time)
        .bind(r.lr_milk_time)
        .bind(r.rf_milk_time)
        .bind(r.rr_milk_time)
        .bind(r.lf_dead_milk_time)
        .bind(r.lr_dead_milk_time)
        .bind(r.rf_dead_milk_time)
        .bind(r.rr_dead_milk_time)
        .bind(r.lf_x_position)
        .bind(r.lf_y_position)
        .bind(r.lfz_position)
        .bind(r.lr_x_position)
        .bind(r.lr_y_position)
        .bind(r.lr_z_position)
        .bind(r.rf_x_position)
        .bind(r.rf_y_position)
        .bind(r.rf_z_position)
        .bind(r.rr_x_position)
        .bind(r.rr_y_position)
        .bind(r.rr_z_position)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_feed_day_amounts(
    pool: &PgPool,
    records: &[FeedDayAmount],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "feed_day_amounts") else {
            continue;
        };
        let Some(date) = parse_lely_date(&r.feed_date) else {
            continue;
        };
        let Some(feed_number) = r.feed_number else {
            continue;
        };
        let total = r.total.unwrap_or(0.0);

        let result = sqlx::query(
            "INSERT INTO feed_day_amounts (animal_id, feed_date, feed_number, total, rest_feed) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (animal_id, feed_date, feed_number) DO UPDATE SET \
             total = EXCLUDED.total, rest_feed = COALESCE(EXCLUDED.rest_feed, feed_day_amounts.rest_feed)"
        )
        .bind(animal_id)
        .bind(date)
        .bind(feed_number)
        .bind(total)
        .bind(r.rest_feed)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_feed_visits(
    pool: &PgPool,
    records: &[FeedVisit],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "feed_visits") else {
            continue;
        };
        let Some(dt) = parse_lely_datetime(&r.feed_date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO feed_visits (animal_id, visit_datetime, feed_number, amount, duration_seconds) \
             VALUES ($1, $2, $3, $4, NULL) \
             ON CONFLICT (animal_id, visit_datetime) DO UPDATE SET \
             amount = COALESCE(EXCLUDED.amount, feed_visits.amount)"
        )
        .bind(animal_id)
        .bind(dt)
        .bind(r.number_of_feed_type)
        .bind(r.intake.map(|i| i as f64))
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_activities(
    pool: &PgPool,
    records: &[Activity],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "activities") else {
            continue;
        };
        let Some(dt) = parse_lely_datetime(&r.activity_date_time) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO activities (animal_id, activity_datetime, activity_counter, heat_attention) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (animal_id, activity_datetime) DO UPDATE SET \
             activity_counter = COALESCE(EXCLUDED.activity_counter, activities.activity_counter), \
             heat_attention = COALESCE(EXCLUDED.heat_attention, activities.heat_attention)"
        )
        .bind(animal_id)
        .bind(dt)
        .bind(r.activity_counter)
        .bind(r.heat_attention)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_ruminations(
    pool: &PgPool,
    records: &[Rumination],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "ruminations") else {
            continue;
        };
        let Some(dt) = parse_lely_datetime(&r.date_time) else {
            continue;
        };
        let date = dt.date_naive();

        let result = sqlx::query(
            "INSERT INTO ruminations (animal_id, date, eating_seconds, rumination_minutes) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (animal_id, date) DO UPDATE SET \
             eating_seconds = COALESCE(EXCLUDED.eating_seconds, ruminations.eating_seconds), \
             rumination_minutes = COALESCE(EXCLUDED.rumination_minutes, ruminations.rumination_minutes)"
        )
        .bind(animal_id)
        .bind(date)
        .bind(r.eating_seconds)
        .bind(r.rumination_minutes)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_grazing_data(pool: &PgPool, records: &[Grazing]) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(date) = parse_lely_date(&r.date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO grazing_data (date, animal_count, pasture_time, lactation_period, \
             total_milking_cows, cows_14dil, percentage_in_pasture, grazing_day_yes_no, \
             sd_time_pasture, cum_pasture_days, cum_total_pasturetime, tank_number) \
             VALUES ($1, $2, $3, NULL, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT (date) DO UPDATE SET \
             animal_count = COALESCE(EXCLUDED.animal_count, grazing_data.animal_count), \
             pasture_time = COALESCE(EXCLUDED.pasture_time, grazing_data.pasture_time), \
             total_milking_cows = COALESCE(EXCLUDED.total_milking_cows, grazing_data.total_milking_cows), \
             cows_14dil = COALESCE(EXCLUDED.cows_14dil, grazing_data.cows_14dil), \
             percentage_in_pasture = COALESCE(EXCLUDED.percentage_in_pasture, grazing_data.percentage_in_pasture), \
             grazing_day_yes_no = COALESCE(EXCLUDED.grazing_day_yes_no, grazing_data.grazing_day_yes_no), \
             sd_time_pasture = COALESCE(EXCLUDED.sd_time_pasture, grazing_data.sd_time_pasture), \
             cum_pasture_days = COALESCE(EXCLUDED.cum_pasture_days, grazing_data.cum_pasture_days), \
             cum_total_pasturetime = COALESCE(EXCLUDED.cum_total_pasturetime, grazing_data.cum_total_pasturetime), \
             tank_number = COALESCE(EXCLUDED.tank_number, grazing_data.tank_number)"
        )
        .bind(date)
        .bind(r.grazing_time)
        .bind(None::<i32>)
        .bind(r.total_milking_cows)
        .bind(r.cows14_dil)
        .bind(r.percentage_in_pasture)
        .bind(r.grazing_day_yes_no)
        .bind(r.sd_time_pasture)
        .bind(r.cum_pasture_days)
        .bind(r.cum_total_pasturetime)
        .bind(r.tank_number)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_calvings(
    pool: &PgPool,
    records: &[Calving],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "calvings") else {
            continue;
        };
        let Some(date) = parse_lely_date(&r.calving_date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO calvings (animal_id, calving_date, remarks, lac_number) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT DO NOTHING",
        )
        .bind(animal_id)
        .bind(date)
        .bind(&r.remarks)
        .bind(r.lac_number)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_inseminations(
    pool: &PgPool,
    records: &[Insemination],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "inseminations") else {
            continue;
        };
        let Some(date) = parse_lely_date(&r.insemination_date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO inseminations (animal_id, insemination_date, sire_code, insemination_type, charge_number) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT DO NOTHING"
        )
        .bind(animal_id)
        .bind(date)
        .bind(&r.sire_code)
        .bind(&r.insemination_type)
        .bind(&r.charge_number)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_pregnancies(
    pool: &PgPool,
    records: &[Pregnancy],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "pregnancies") else {
            continue;
        };
        let Some(date) = parse_lely_date(&r.pregnancy_date) else {
            continue;
        };
        let ins_date = parse_lely_date(&r.insemination_date);

        let result = sqlx::query(
            "INSERT INTO pregnancies (animal_id, pregnancy_date, pregnancy_type, insemination_date) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT DO NOTHING"
        )
        .bind(animal_id)
        .bind(date)
        .bind(&r.pregnancy_type)
        .bind(ins_date)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_heats(
    pool: &PgPool,
    records: &[Heat],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "heats") else {
            continue;
        };
        let Some(date) = parse_lely_date(&r.heat_date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO heats (animal_id, heat_date) \
             VALUES ($1, $2) \
             ON CONFLICT DO NOTHING",
        )
        .bind(animal_id)
        .bind(date)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_dry_offs(
    pool: &PgPool,
    records: &[DryOff],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "dry_offs") else {
            continue;
        };
        let Some(date) = parse_lely_date(&r.dry_off_date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO dry_offs (animal_id, dry_off_date) \
             VALUES ($1, $2) \
             ON CONFLICT DO NOTHING",
        )
        .bind(animal_id)
        .bind(date)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_sires(pool: &PgPool, records: &[Sire]) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(code) = &r.sire_code else { continue };
        let Some(ln) = &r.life_number else { continue };

        let result = sqlx::query(
            "INSERT INTO sires (sire_code, life_number, name, active) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT DO NOTHING",
        )
        .bind(code)
        .bind(ln)
        .bind(&r.sire_name)
        .bind(r.sire_active.unwrap_or(true))
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_transfers(
    pool: &PgPool,
    records: &[Transfer],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "transfers") else {
            continue;
        };
        let Some(dt) = parse_lely_datetime(&r.transfer_date) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO transfers (animal_id, transfer_date, transfer_type, reason_id, from_location, to_location) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT DO NOTHING"
        )
        .bind(animal_id)
        .bind(dt)
        .bind(&r.transfer_type)
        .bind(r.reason_id)
        .bind(&r.ucn_origin)
        .bind(&r.ucn_destination)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

pub async fn upsert_bloodlines(
    pool: &PgPool,
    records: &[BloodLine],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut count = 0u64;
    for r in records {
        let Some(animal_id) = cache.resolve_or_warn(&r.life_number, "bloodlines") else {
            continue;
        };
        let Some(code) = &r.blood_type_code else {
            continue;
        };
        let pct = r.percentage.unwrap_or(0.0);

        let result = sqlx::query(
            "INSERT INTO bloodlines (animal_id, blood_type_code, percentage) \
             VALUES ($1, $2, $3) \
             ON CONFLICT DO NOTHING",
        )
        .bind(animal_id)
        .bind(code)
        .bind(pct)
        .execute(pool)
        .await?;

        count += result.rows_affected();
    }
    Ok(count)
}

fn parse_lely_date(s: &Option<String>) -> Option<NaiveDate> {
    s.as_ref().and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(d)
            .map(|dt| dt.date_naive())
            .or_else(|_| NaiveDate::parse_from_str(d, "%Y-%m-%d"))
            .ok()
    })
}

fn parse_lely_datetime(s: &Option<String>) -> Option<DateTime<Utc>> {
    s.as_ref().and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(d)
            .ok()
            .map(|dt| dt.to_utc())
    })
}
