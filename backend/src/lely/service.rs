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

const BATCH_SIZE: usize = 500;

pub async fn upsert_animals(
    pool: &PgPool,
    records: &[AnimalResponse],
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut life_numbers = Vec::with_capacity(chunk.len());
        let mut names = Vec::with_capacity(chunk.len());
        let mut user_numbers = Vec::with_capacity(chunk.len());
        let mut genders = Vec::with_capacity(chunk.len());
        let mut birth_dates = Vec::with_capacity(chunk.len());
        let mut hair_colors = Vec::with_capacity(chunk.len());
        let mut father_lns = Vec::with_capacity(chunk.len());
        let mut mother_lns = Vec::with_capacity(chunk.len());
        let mut descriptions = Vec::with_capacity(chunk.len());
        let mut ucn_numbers = Vec::with_capacity(chunk.len());
        let mut use_as_sires = Vec::with_capacity(chunk.len());
        let mut locations = Vec::with_capacity(chunk.len());
        let mut group_numbers = Vec::with_capacity(chunk.len());
        let mut keeps = Vec::with_capacity(chunk.len());
        let mut gestations = Vec::with_capacity(chunk.len());
        let mut responder_numbers = Vec::with_capacity(chunk.len());

        for r in chunk {
            let Some(ln) = &r.life_number else { continue };
            let Some(bd) = parse_lely_date(&r.birth_date) else {
                tracing::warn!(life_number = %ln, "Животное без даты рождения, пропущено");
                continue;
            };
            let g = match r.gender.as_deref() {
                Some("Male") => "male",
                Some("Female") => "female",
                _ => "female",
            };
            life_numbers.push(ln.clone());
            names.push(r.name.clone());
            user_numbers.push(r.user_number);
            genders.push(g.to_string());
            birth_dates.push(bd);
            hair_colors.push(r.hair_color_code.clone());
            father_lns.push(r.father_life_number.clone());
            mother_lns.push(r.mother_life_number.clone());
            descriptions.push(r.description.clone());
            ucn_numbers.push(r.ucn_number.clone());
            use_as_sires.push(r.use_as_sire);
            locations.push(r.location.clone());
            group_numbers.push(r.group_number);
            keeps.push(r.keep);
            gestations.push(r.gestation);
            responder_numbers.push(r.responder_number.clone());
        }

        if life_numbers.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO animals (life_number, name, user_number, gender, birth_date, hair_color_code, \
             father_life_number, mother_life_number, description, ucn_number, use_as_sire, location, \
             group_number, keep, gestation, responder_number, active, updated_at) \
             SELECT * FROM unnest(
               $1::text[], $2::text[], $3::bigint[], $4::gender_type[], $5::date[], $6::text[],
               $7::text[], $8::text[], $9::text[], $10::text[], $11::boolean[], $12::text[],
               $13::int[], $14::boolean[], $15::int[], $16::text[],
               array_fill(true, array[array_length($1::text[], 1)]),
               array_fill(NOW(), array[array_length($1::text[], 1)])
             ) \
             ON CONFLICT DO NOTHING"
        )
        .bind(&life_numbers)
        .bind(&names)
        .bind(&user_numbers)
        .bind(&genders)
        .bind(&birth_dates)
        .bind(&hair_colors)
        .bind(&father_lns)
        .bind(&mother_lns)
        .bind(&descriptions)
        .bind(&ucn_numbers)
        .bind(&use_as_sires)
        .bind(&locations)
        .bind(&group_numbers)
        .bind(&keeps)
        .bind(&gestations)
        .bind(&responder_numbers)
        .execute(pool)
        .await?;

        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_milk_day_productions(
    pool: &PgPool,
    records: &[DayProduction],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dates = Vec::new();
        let mut milk_amounts = Vec::new();
        let mut avg_amounts = Vec::new();
        let mut avg_weights = Vec::new();
        let mut isks = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "milk_day_productions") else { continue };
            let Some(d) = parse_lely_date(&r.date) else { continue };
            animal_ids.push(aid);
            dates.push(d);
            milk_amounts.push(r.milk_day_production);
            avg_amounts.push(r.milk_day_production_average);
            avg_weights.push(r.average_weight);
            isks.push(r.isk);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO milk_day_productions (animal_id, date, milk_amount, avg_amount, avg_weight, isk) \
             SELECT * FROM unnest($1::int[], $2::date[], $3::float8[], $4::float8[], $5::float8[], $6::float8[]) \
             ON CONFLICT (animal_id, date) DO UPDATE SET \
             milk_amount = COALESCE(EXCLUDED.milk_amount, milk_day_productions.milk_amount), \
             avg_amount = COALESCE(EXCLUDED.avg_amount, milk_day_productions.avg_amount), \
             avg_weight = COALESCE(EXCLUDED.avg_weight, milk_day_productions.avg_weight), \
             isk = COALESCE(EXCLUDED.isk, milk_day_productions.isk)"
        )
        .bind(&animal_ids).bind(&dates).bind(&milk_amounts).bind(&avg_amounts).bind(&avg_weights).bind(&isks)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_milk_visits(
    pool: &PgPool,
    records: &[MilkVisit],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dts = Vec::new();
        let mut yields = Vec::new();
        let mut bottles = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "milk_visits") else { continue };
            let Some(dt) = parse_lely_datetime(&r.milking_date) else { continue };
            animal_ids.push(aid);
            dts.push(dt);
            yields.push(r.milk_yield);
            bottles.push(r.bottle_number);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO milk_visits (animal_id, visit_datetime, milk_amount, duration_seconds, milk_destination) \
             SELECT * FROM unnest($1::int[], $2::timestamptz[], $3::float8[], array_fill(null::int, array[array_length($1::int[], 1)]), $4::int[]) \
             ON CONFLICT (animal_id, visit_datetime) DO UPDATE SET \
             milk_amount = COALESCE(EXCLUDED.milk_amount, milk_visits.milk_amount), \
             milk_destination = COALESCE(EXCLUDED.milk_destination, milk_visits.milk_destination)"
        )
        .bind(&animal_ids).bind(&dts).bind(&yields).bind(&bottles)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_milk_visit_quality(
    pool: &PgPool,
    records: &[MilkVisitQuality],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dts = Vec::new();
        let mut start_dts = Vec::new();
        let mut devices = Vec::new();
        let mut successes = Vec::new();
        let mut yields = Vec::new();
        let mut bottles = Vec::new();
        let mut temps = Vec::new();
        let mut weights = Vec::new();
        let mut destinations = Vec::new();
        let mut lf_col = Vec::new();
        let mut lr_col = Vec::new();
        let mut rf_col = Vec::new();
        let mut rr_col = Vec::new();
        let mut lf_cond = Vec::new();
        let mut lr_cond = Vec::new();
        let mut rf_cond = Vec::new();
        let mut rr_cond = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "milk_visit_quality") else { continue };
            let Some(dt) = parse_lely_datetime(&r.milking_date) else { continue };
            animal_ids.push(aid);
            dts.push(dt);
            start_dts.push(parse_lely_datetime(&r.milking_start_date));
            devices.push(r.device_address);
            successes.push(r.success_milking);
            yields.push(r.milk_yield);
            bottles.push(r.bottle_number);
            temps.push(r.milk_temperature);
            weights.push(r.weight);
            destinations.push(r.milk_destination);
            lf_col.push(r.lf_colour_code.clone());
            lr_col.push(r.lr_colour_code.clone());
            rf_col.push(r.rf_colour_code.clone());
            rr_col.push(r.rr_colour_code.clone());
            lf_cond.push(r.lf_conductivity);
            lr_cond.push(r.lr_conductivity);
            rf_cond.push(r.rf_conductivity);
            rr_cond.push(r.rr_conductivity);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO milk_visit_quality (animal_id, visit_datetime, milking_start_date, device_address, \
             success_milking, milk_yield, bottle_number, milk_temperature, weight, milk_destination, \
             lf_colour_code, lr_colour_code, rf_colour_code, rr_colour_code, \
             lf_conductivity, lr_conductivity, rf_conductivity, rr_conductivity) \
             SELECT * FROM unnest($1::int[], $2::timestamptz[], $3::timestamptz[], $4::int[], \
             $5::boolean[], $6::float8[], $7::int[], $8::float8[], $9::int[], $10::int[], \
             $11::text[], $12::text[], $13::text[], $14::text[], \
             $15::int[], $16::int[], $17::int[], $18::int[]) \
             ON CONFLICT (animal_id, visit_datetime) DO UPDATE SET \
             milk_temperature = COALESCE(EXCLUDED.milk_temperature, milk_visit_quality.milk_temperature), \
             lf_conductivity = COALESCE(EXCLUDED.lf_conductivity, milk_visit_quality.lf_conductivity), \
             lr_conductivity = COALESCE(EXCLUDED.lr_conductivity, milk_visit_quality.lr_conductivity), \
             rf_conductivity = COALESCE(EXCLUDED.rf_conductivity, milk_visit_quality.rf_conductivity), \
             rr_conductivity = COALESCE(EXCLUDED.rr_conductivity, milk_visit_quality.rr_conductivity)"
        )
        .bind(&animal_ids).bind(&dts).bind(&start_dts).bind(&devices)
        .bind(&successes).bind(&yields).bind(&bottles).bind(&temps).bind(&weights).bind(&destinations)
        .bind(&lf_col).bind(&lr_col).bind(&rf_col).bind(&rr_col)
        .bind(&lf_cond).bind(&lr_cond).bind(&rf_cond).bind(&rr_cond)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_milk_day_quality(
    pool: &PgPool,
    records: &[DayProductionQuality],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dates = Vec::new();
        let mut milk_amounts = Vec::new();
        let mut avg_amounts = Vec::new();
        let mut avg_weights = Vec::new();
        let mut isks = Vec::new();
        let mut fats = Vec::new();
        let mut proteins = Vec::new();
        let mut lactoses = Vec::new();
        let mut sccs = Vec::new();
        let mut milkings = Vec::new();
        let mut refusals = Vec::new();
        let mut failures = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "milk_day_quality") else { continue };
            let Some(d) = parse_lely_date(&r.date) else { continue };
            animal_ids.push(aid);
            dates.push(d);
            milk_amounts.push(r.milk_day_production);
            avg_amounts.push(r.milk_day_production_average);
            avg_weights.push(r.average_weight);
            isks.push(r.isk);
            fats.push(r.fat_percentage);
            proteins.push(r.protein_percentage);
            lactoses.push(r.lactose_percentage);
            sccs.push(r.scc);
            milkings.push(r.mdp_milkings);
            refusals.push(r.mdp_refusals);
            failures.push(r.mdp_failures);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO milk_quality (animal_id, date, milk_amount, avg_amount, avg_weight, isk, \
             fat_percentage, protein_percentage, lactose_percentage, scc, milkings, refusals, failures) \
             SELECT * FROM unnest($1::int[], $2::date[], $3::float8[], $4::float8[], $5::float8[], $6::float8[], \
             $7::float8[], $8::float8[], $9::float8[], $10::int[], $11::int[], $12::int[], $13::int[]) \
             ON CONFLICT (animal_id, date) DO UPDATE SET \
             fat_percentage = COALESCE(EXCLUDED.fat_percentage, milk_quality.fat_percentage), \
             protein_percentage = COALESCE(EXCLUDED.protein_percentage, milk_quality.protein_percentage), \
             lactose_percentage = COALESCE(EXCLUDED.lactose_percentage, milk_quality.lactose_percentage), \
             scc = COALESCE(EXCLUDED.scc, milk_quality.scc), \
             milkings = COALESCE(EXCLUDED.milkings, milk_quality.milkings), \
             refusals = COALESCE(EXCLUDED.refusals, milk_quality.refusals), \
             failures = COALESCE(EXCLUDED.failures, milk_quality.failures)"
        )
        .bind(&animal_ids).bind(&dates).bind(&milk_amounts).bind(&avg_amounts).bind(&avg_weights).bind(&isks)
        .bind(&fats).bind(&proteins).bind(&lactoses).bind(&sccs).bind(&milkings).bind(&refusals).bind(&failures)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_robot_data(
    pool: &PgPool,
    records: &[RobotData],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dts = Vec::new();
        let mut devices = Vec::new();
        let mut speeds = Vec::new();
        let mut speeds_max = Vec::new();
        let mut lf_t = Vec::new(); let mut lr_t = Vec::new(); let mut rf_t = Vec::new(); let mut rr_t = Vec::new();
        let mut lf_dt = Vec::new(); let mut lr_dt = Vec::new(); let mut rf_dt = Vec::new(); let mut rr_dt = Vec::new();
        let mut lf_x = Vec::new(); let mut lf_y = Vec::new(); let mut lf_z = Vec::new();
        let mut lr_x = Vec::new(); let mut lr_y = Vec::new(); let mut lr_z = Vec::new();
        let mut rf_x = Vec::new(); let mut rf_y = Vec::new(); let mut rf_z = Vec::new();
        let mut rr_x = Vec::new(); let mut rr_y = Vec::new(); let mut rr_z = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "robot_milk_data") else { continue };
            let Some(dt) = parse_lely_datetime(&r.milking_date) else { continue };
            animal_ids.push(aid);
            dts.push(dt);
            devices.push(r.device_address);
            speeds.push(r.milk_speed);
            speeds_max.push(r.milk_speed_max);
            lf_t.push(r.lf_milk_time); lr_t.push(r.lr_milk_time); rf_t.push(r.rf_milk_time); rr_t.push(r.rr_milk_time);
            lf_dt.push(r.lf_dead_milk_time); lr_dt.push(r.lr_dead_milk_time); rf_dt.push(r.rf_dead_milk_time); rr_dt.push(r.rr_dead_milk_time);
            lf_x.push(r.lf_x_position); lf_y.push(r.lf_y_position); lf_z.push(r.lfz_position);
            lr_x.push(r.lr_x_position); lr_y.push(r.lr_y_position); lr_z.push(r.lr_z_position);
            rf_x.push(r.rf_x_position); rf_y.push(r.rf_y_position); rf_z.push(r.rf_z_position);
            rr_x.push(r.rr_x_position); rr_y.push(r.rr_y_position); rr_z.push(r.rr_z_position);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO robot_milk_data (animal_id, milking_date, device_address, milk_speed, milk_speed_max, \
             lf_milk_time, lr_milk_time, rf_milk_time, rr_milk_time, \
             lf_dead_milk_time, lr_dead_milk_time, rf_dead_milk_time, rr_dead_milk_time, \
             lf_x_position, lf_y_position, lf_z_position, \
             lr_x_position, lr_y_position, lr_z_position, \
             rf_x_position, rf_y_position, rf_z_position, \
             rr_x_position, rr_y_position, rr_z_position) \
             SELECT * FROM unnest($1::int[], $2::timestamptz[], $3::int[], $4::float8[], $5::float8[], \
             $6::int[], $7::int[], $8::int[], $9::int[], $10::int[], $11::int[], $12::int[], $13::int[], \
             $14::int[], $15::int[], $16::int[], $17::int[], $18::int[], $19::int[], \
             $20::int[], $21::int[], $22::int[], $23::int[], $24::int[], $25::int[]) \
             ON CONFLICT (animal_id, milking_date) DO UPDATE SET \
             milk_speed = COALESCE(EXCLUDED.milk_speed, robot_milk_data.milk_speed), \
             milk_speed_max = COALESCE(EXCLUDED.milk_speed_max, robot_milk_data.milk_speed_max)"
        )
        .bind(&animal_ids).bind(&dts).bind(&devices).bind(&speeds).bind(&speeds_max)
        .bind(&lf_t).bind(&lr_t).bind(&rf_t).bind(&rr_t)
        .bind(&lf_dt).bind(&lr_dt).bind(&rf_dt).bind(&rr_dt)
        .bind(&lf_x).bind(&lf_y).bind(&lf_z)
        .bind(&lr_x).bind(&lr_y).bind(&lr_z)
        .bind(&rf_x).bind(&rf_y).bind(&rf_z)
        .bind(&rr_x).bind(&rr_y).bind(&rr_z)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_feed_day_amounts(
    pool: &PgPool,
    records: &[FeedDayAmount],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dates = Vec::new();
        let mut feed_numbers = Vec::new();
        let mut totals = Vec::new();
        let mut rest_feeds = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "feed_day_amounts") else { continue };
            let Some(d) = parse_lely_date(&r.feed_date) else { continue };
            let Some(fn_) = r.feed_number else { continue };
            animal_ids.push(aid);
            dates.push(d);
            feed_numbers.push(fn_);
            totals.push(r.total.unwrap_or(0.0));
            rest_feeds.push(r.rest_feed);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO feed_day_amounts (animal_id, feed_date, feed_number, total, rest_feed) \
             SELECT * FROM unnest($1::int[], $2::date[], $3::int[], $4::float8[], $5::int[]) \
             ON CONFLICT (animal_id, feed_date, feed_number) DO UPDATE SET \
             total = EXCLUDED.total, rest_feed = COALESCE(EXCLUDED.rest_feed, feed_day_amounts.rest_feed)"
        )
        .bind(&animal_ids).bind(&dates).bind(&feed_numbers).bind(&totals).bind(&rest_feeds)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_feed_visits(
    pool: &PgPool,
    records: &[FeedVisit],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dts = Vec::new();
        let mut feed_numbers = Vec::new();
        let mut amounts = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "feed_visits") else { continue };
            let Some(dt) = parse_lely_datetime(&r.feed_date) else { continue };
            animal_ids.push(aid);
            dts.push(dt);
            feed_numbers.push(r.number_of_feed_type);
            amounts.push(r.intake.map(|i| i as f64));
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO feed_visits (animal_id, visit_datetime, feed_number, amount, duration_seconds) \
             SELECT * FROM unnest($1::int[], $2::timestamptz[], $3::int[], $4::float8[], array_fill(null::int, array[array_length($1::int[], 1)])) \
             ON CONFLICT (animal_id, visit_datetime) DO UPDATE SET \
             amount = COALESCE(EXCLUDED.amount, feed_visits.amount)"
        )
        .bind(&animal_ids).bind(&dts).bind(&feed_numbers).bind(&amounts)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_activities(
    pool: &PgPool,
    records: &[Activity],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dts = Vec::new();
        let mut counters = Vec::new();
        let mut heats = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "activities") else { continue };
            let Some(dt) = parse_lely_datetime(&r.activity_date_time) else { continue };
            animal_ids.push(aid);
            dts.push(dt);
            counters.push(r.activity_counter);
            heats.push(r.heat_attention);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO activities (animal_id, activity_datetime, activity_counter, heat_attention) \
             SELECT * FROM unnest($1::int[], $2::timestamptz[], $3::int[], $4::boolean[]) \
             ON CONFLICT (animal_id, activity_datetime) DO UPDATE SET \
             activity_counter = COALESCE(EXCLUDED.activity_counter, activities.activity_counter), \
             heat_attention = COALESCE(EXCLUDED.heat_attention, activities.heat_attention)"
        )
        .bind(&animal_ids).bind(&dts).bind(&counters).bind(&heats)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_ruminations(
    pool: &PgPool,
    records: &[Rumination],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dates = Vec::new();
        let mut eating_secs = Vec::new();
        let mut rum_mins = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "ruminations") else { continue };
            let Some(dt) = parse_lely_datetime(&r.date_time) else { continue };
            animal_ids.push(aid);
            dates.push(dt.date_naive());
            eating_secs.push(r.eating_seconds);
            rum_mins.push(r.rumination_minutes);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO ruminations (animal_id, date, eating_seconds, rumination_minutes) \
             SELECT * FROM unnest($1::int[], $2::date[], $3::int[], $4::int[]) \
             ON CONFLICT (animal_id, date) DO UPDATE SET \
             eating_seconds = COALESCE(EXCLUDED.eating_seconds, ruminations.eating_seconds), \
             rumination_minutes = COALESCE(EXCLUDED.rumination_minutes, ruminations.rumination_minutes)"
        )
        .bind(&animal_ids).bind(&dates).bind(&eating_secs).bind(&rum_mins)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_grazing_data(pool: &PgPool, records: &[Grazing]) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut dates = Vec::new();
        let mut grazing_times = Vec::new();
        let mut total_cows = Vec::new();
        let mut cows14 = Vec::new();
        let mut pcts = Vec::new();
        let mut yes_nos = Vec::new();
        let mut sd_times = Vec::new();
        let mut cum_days = Vec::new();
        let mut cum_times = Vec::new();
        let mut tanks = Vec::new();

        for r in chunk {
            let Some(d) = parse_lely_date(&r.date) else { continue };
            dates.push(d);
            grazing_times.push(r.grazing_time);
            total_cows.push(r.total_milking_cows);
            cows14.push(r.cows14_dil);
            pcts.push(r.percentage_in_pasture);
            yes_nos.push(r.grazing_day_yes_no);
            sd_times.push(r.sd_time_pasture);
            cum_days.push(r.cum_pasture_days);
            cum_times.push(r.cum_total_pasturetime);
            tanks.push(r.tank_number);
        }
        if dates.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO grazing_data (date, pasture_time, lactation_period, \
             total_milking_cows, cows_14dil, percentage_in_pasture, grazing_day_yes_no, \
             sd_time_pasture, cum_pasture_days, cum_total_pasturetime, tank_number) \
             SELECT * FROM unnest($1::date[], $2::int[], array_fill(null::text, array[array_length($1::date[], 1)]), \
             $3::int[], $4::int[], $5::float8[], $6::boolean[], $7::int[], $8::int[], $9::int[], $10::bigint[]) \
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
        .bind(&dates).bind(&grazing_times).bind(&total_cows).bind(&cows14).bind(&pcts)
        .bind(&yes_nos).bind(&sd_times).bind(&cum_days).bind(&cum_times).bind(&tanks)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_calvings(
    pool: &PgPool,
    records: &[Calving],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dates = Vec::new();
        let mut remarks = Vec::new();
        let mut lac_numbers = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "calvings") else { continue };
            let Some(d) = parse_lely_date(&r.calving_date) else { continue };
            animal_ids.push(aid);
            dates.push(d);
            remarks.push(r.remarks.clone());
            lac_numbers.push(r.lac_number);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO calvings (animal_id, calving_date, remarks, lac_number) \
             SELECT * FROM unnest($1::int[], $2::date[], $3::text[], $4::int[]) \
             ON CONFLICT DO NOTHING"
        )
        .bind(&animal_ids).bind(&dates).bind(&remarks).bind(&lac_numbers)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_inseminations(
    pool: &PgPool,
    records: &[Insemination],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dates = Vec::new();
        let mut sire_codes = Vec::new();
        let mut ins_types = Vec::new();
        let mut charge_numbers = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "inseminations") else { continue };
            let Some(d) = parse_lely_date(&r.insemination_date) else { continue };
            animal_ids.push(aid);
            dates.push(d);
            sire_codes.push(r.sire_code.clone());
            ins_types.push(r.insemination_type.clone());
            charge_numbers.push(r.charge_number.clone());
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO inseminations (animal_id, insemination_date, sire_code, insemination_type, charge_number) \
             SELECT * FROM unnest($1::int[], $2::date[], $3::text[], $4::text[], $5::text[]) \
             ON CONFLICT DO NOTHING"
        )
        .bind(&animal_ids).bind(&dates).bind(&sire_codes).bind(&ins_types).bind(&charge_numbers)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_pregnancies(
    pool: &PgPool,
    records: &[Pregnancy],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dates = Vec::new();
        let mut p_types = Vec::new();
        let mut ins_dates = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "pregnancies") else { continue };
            let Some(d) = parse_lely_date(&r.pregnancy_date) else { continue };
            animal_ids.push(aid);
            dates.push(d);
            p_types.push(r.pregnancy_type.clone());
            ins_dates.push(parse_lely_date(&r.insemination_date));
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO pregnancies (animal_id, pregnancy_date, pregnancy_type, insemination_date) \
             SELECT * FROM unnest($1::int[], $2::date[], $3::text[], $4::date[]) \
             ON CONFLICT DO NOTHING"
        )
        .bind(&animal_ids).bind(&dates).bind(&p_types).bind(&ins_dates)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_heats(
    pool: &PgPool,
    records: &[Heat],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dates = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "heats") else { continue };
            let Some(d) = parse_lely_date(&r.heat_date) else { continue };
            animal_ids.push(aid);
            dates.push(d);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO heats (animal_id, heat_date) \
             SELECT * FROM unnest($1::int[], $2::date[]) \
             ON CONFLICT DO NOTHING"
        )
        .bind(&animal_ids).bind(&dates)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_dry_offs(
    pool: &PgPool,
    records: &[DryOff],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dates = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "dry_offs") else { continue };
            let Some(d) = parse_lely_date(&r.dry_off_date) else { continue };
            animal_ids.push(aid);
            dates.push(d);
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO dry_offs (animal_id, dry_off_date) \
             SELECT * FROM unnest($1::int[], $2::date[]) \
             ON CONFLICT DO NOTHING"
        )
        .bind(&animal_ids).bind(&dates)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_sires(pool: &PgPool, records: &[Sire]) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut codes = Vec::new();
        let mut lns = Vec::new();
        let mut names = Vec::new();
        let mut actives = Vec::new();

        for r in chunk {
            let Some(c) = &r.sire_code else { continue };
            let Some(ln) = &r.life_number else { continue };
            codes.push(c.clone());
            lns.push(ln.clone());
            names.push(r.sire_name.clone());
            actives.push(r.sire_active.unwrap_or(true));
        }
        if codes.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO sires (sire_code, life_number, name, active) \
             SELECT * FROM unnest($1::text[], $2::text[], $3::text[], $4::boolean[]) \
             ON CONFLICT DO NOTHING"
        )
        .bind(&codes).bind(&lns).bind(&names).bind(&actives)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_transfers(
    pool: &PgPool,
    records: &[Transfer],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut dts = Vec::new();
        let mut types_ = Vec::new();
        let mut reason_ids = Vec::new();
        let mut from_locs = Vec::new();
        let mut to_locs = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "transfers") else { continue };
            let Some(dt) = parse_lely_datetime(&r.transfer_date) else { continue };
            animal_ids.push(aid);
            dts.push(dt);
            types_.push(r.transfer_type.clone());
            reason_ids.push(r.reason_id);
            from_locs.push(r.ucn_origin.clone());
            to_locs.push(r.ucn_destination.clone());
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO transfers (animal_id, transfer_date, transfer_type, reason_id, from_location, to_location) \
             SELECT * FROM unnest($1::int[], $2::timestamptz[], $3::text[], $4::int[], $5::text[], $6::text[]) \
             ON CONFLICT DO NOTHING"
        )
        .bind(&animal_ids).bind(&dts).bind(&types_).bind(&reason_ids).bind(&from_locs).bind(&to_locs)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
}

pub async fn upsert_bloodlines(
    pool: &PgPool,
    records: &[BloodLine],
    cache: &AnimalCache,
) -> Result<u64, anyhow::Error> {
    let mut total = 0u64;
    for chunk in records.chunks(BATCH_SIZE) {
        let mut animal_ids = Vec::new();
        let mut codes = Vec::new();
        let mut pcts = Vec::new();

        for r in chunk {
            let Some(aid) = cache.resolve_or_warn(&r.life_number, "bloodlines") else { continue };
            let Some(c) = &r.blood_type_code else { continue };
            animal_ids.push(aid);
            codes.push(c.clone());
            pcts.push(r.percentage.unwrap_or(0.0));
        }
        if animal_ids.is_empty() { continue; }

        let result = sqlx::query(
            "INSERT INTO bloodlines (animal_id, blood_type_code, percentage) \
             SELECT * FROM unnest($1::int[], $2::text[], $3::float8[]) \
             ON CONFLICT DO NOTHING"
        )
        .bind(&animal_ids).bind(&codes).bind(&pcts)
        .execute(pool).await?;
        total += result.rows_affected();
    }
    Ok(total)
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
