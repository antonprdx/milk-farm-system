use std::sync::LazyLock;

use chrono::{Datelike, Duration, NaiveDate};
use rand::Rng;
use rand::SeedableRng;
use rand::distr::Distribution;
use rand::seq::IndexedRandom;
use sqlx::PgPool;

use super::names::*;

struct Normal {
    mean: f64,
    stddev: f64,
}

impl Normal {
    fn new(mean: f64, stddev: f64) -> Self {
        Self { mean, stddev }
    }
}

impl Distribution<f64> for Normal {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let u1: f64 = rng.random();
        let u2: f64 = rng.random();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        self.mean + self.stddev * z
    }
}

fn normal_range(rng: &mut impl Rng, mean: f64, stddev: f64, min: f64, max: f64) -> f64 {
    let v = Normal::new(mean, stddev).sample(rng);
    v.clamp(min, max)
}

fn pick<'a, T>(rng: &mut impl Rng, slice: &'a [T]) -> &'a T {
    slice.choose(rng).unwrap()
}

fn pick_n<T: Clone>(rng: &mut impl Rng, slice: &[T], n: usize) -> Vec<T> {
    let mut copy: Vec<&T> = slice.iter().collect();
    let mut result = Vec::with_capacity(n.min(copy.len()));
    for _ in 0..n.min(copy.len()) {
        let idx = rng.random_range(0..copy.len());
        result.push(copy[idx].clone());
        copy.swap_remove(idx);
    }
    result
}

fn fmt_phone(rng: &mut impl Rng) -> String {
    format!(
        "{} {:03}-{:02}-{:02}",
        pick(rng, PHONE_CODES),
        rng.random_range(100..999u32),
        rng.random_range(10..99u32),
        rng.random_range(10..99u32)
    )
}

fn fmt_life_number(rng: &mut impl Rng) -> String {
    format!(
        "RU{}{:012}",
        rng.random_range(0..999u32),
        rng.random_range(0u64..999999999999u64)
    )
}

static GLOBAL_RNG: LazyLock<std::sync::Mutex<rand::rngs::SmallRng>> =
    LazyLock::new(|| std::sync::Mutex::new(rand::rngs::SmallRng::from_os_rng()));

fn wood_milk(day_in_lactation: i64, lac_number: i32, rng: &mut impl Rng) -> f64 {
    let b = 0.2_f64;
    let c = 0.0035_f64;
    let peak = match lac_number {
        1 => normal_range(rng, 25.0, 3.0, 18.0, 32.0),
        2 => normal_range(rng, 30.0, 3.5, 22.0, 38.0),
        _ => normal_range(rng, 33.0, 4.0, 24.0, 42.0),
    };
    let t = day_in_lactation as f64;
    if t < 1.0 {
        return 0.0;
    }
    let a = peak / (t.powf(b) * (-c * t).exp());
    let base = a * t.powf(b) * (-c * t).exp();
    let noise = normal_range(rng, 1.0, 0.08, 0.7, 1.3);
    (base * noise).max(0.0).round() / 10.0
}

pub struct SeedConfig {
    pub num_cows: usize,
    pub num_years: i64,
}

pub struct AnimalInfo {
    pub id: i32,
    pub gender: String,
    pub birth_date: NaiveDate,
    pub active: bool,
    pub location: Option<String>,
    pub group_number: Option<i32>,
}

pub struct LactationInfo {
    pub animal_id: i32,
    pub calving_date: NaiveDate,
    pub lac_number: i32,
    pub dry_off_date: Option<NaiveDate>,
    pub next_calving_date: Option<NaiveDate>,
}

pub async fn seed_locations(pool: &PgPool) -> Vec<i32> {
    let mut ids = Vec::new();
    for i in 0..LOCATIONS.len() {
        let row = sqlx::query_as::<_, (i32,)>(
            "INSERT INTO locations (name, location_type) VALUES ($1, $2) RETURNING id",
        )
        .bind(LOCATIONS[i])
        .bind(
            LOCATION_TYPES
                .get(i)
                .map(|_| LOCATION_TYPES[i.min(LOCATION_TYPES.len() - 1)]),
        )
        .fetch_one(pool)
        .await
        .unwrap();
        ids.push(row.0);
    }
    println!("  locations: {} rows", ids.len());
    ids
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_contacts(pool: &PgPool) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let count = rng.random_range(30..50);
    let mut values = Vec::new();
    for i in 0..count {
        let first = pick(&mut rng, FIRST_NAMES);
        let last = pick(&mut rng, LAST_NAMES);
        let patronymic = pick(&mut rng, PATRONYMICS);
        let name = format!("{} {} {}", last, first, patronymic);
        let ct = pick(&mut rng, CONTACT_TYPES);
        let company = if rng.random_bool(0.6) {
            pick(&mut rng, COMPANIES)
        } else {
            ""
        };
        let phone = fmt_phone(&mut rng);
        let email = format!(
            "{}.{}@farm{}.ru",
            first.to_lowercase(),
            last.to_lowercase(),
            i % 5 + 1
        );
        let city = pick(&mut rng, CITIES);
        let street = pick(&mut rng, STREETS);
        let building = rng.random_range(1..150u32);
        let active = rng.random_bool(0.9);
        values.push(format!(
            "('{}', '{}', '{}', '{}', '{}', '{}', {})",
            name.replace('\'', "''"),
            ct.replace('\'', "''"),
            company.replace('\'', "''"),
            phone,
            email,
            format!("{}, {} д.{}", city, street, building).replace('\'', "''"),
            active,
        ));
    }
    let sql = format!(
        "INSERT INTO contacts (name, contact_type_name, company_name, phone_cell, email, street_name, active) VALUES {}",
        values.join(", ")
    );
    sqlx::query(&sql).execute(pool).await.unwrap();
    println!("  contacts: {} rows", count);
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_sires(pool: &PgPool) -> Vec<String> {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let count = rng.random_range(20..30);
    let mut sire_codes = Vec::new();
    let mut values = Vec::new();
    for i in 0..count {
        let code = format!("SIRE{:04}", i + 1);
        let ln = fmt_life_number(&mut rng);
        let name = pick(&mut rng, BULL_NAMES);
        sire_codes.push(code.clone());
        values.push(format!("('{}', '{}', '{}')", code, ln, name));
    }
    let sql = format!(
        "INSERT INTO sires (sire_code, life_number, name) VALUES {}",
        values.join(", ")
    );
    sqlx::query(&sql).execute(pool).await.unwrap();
    println!("  sires: {} rows", count);
    sire_codes
}

pub async fn seed_feed_types(pool: &PgPool) -> Vec<i32> {
    let mut ids = Vec::new();
    let dry_matters = [
        85.0, 45.0, 30.0, 88.0, 89.0, 90.0, 95.0, 42.0, 86.0, 87.0, 92.0, 55.0,
    ];
    let prices = [
        5.0, 4.5, 3.5, 12.0, 14.0, 11.0, 18.0, 4.8, 2.5, 10.0, 8.0, 3.8,
    ];
    for i in 0..FEED_TYPE_NAMES.len() {
        let row = sqlx::query_as::<_, (i32,)>(
			"INSERT INTO feed_types (number_of_feed_type, feed_type, name, dry_matter_percentage, price) VALUES ($1, $2, $3, $4, $5) RETURNING id",
		)
		.bind((i + 1) as i32)
		.bind(FEED_TYPE_CODES[i])
		.bind(FEED_TYPE_NAMES[i])
		.bind(dry_matters[i])
		.bind(prices[i])
		.fetch_one(pool)
		.await
		.unwrap();
        ids.push(row.0);
    }
    println!("  feed_types: {} rows", ids.len());
    ids
}

pub async fn seed_feed_groups(pool: &PgPool) -> Vec<i32> {
    let mins = [0.0, 30.0, 20.0, 0.0, 0.0, 15.0];
    let maxs = [35.0, 50.0, 30.0, 20.0, 0.0, 35.0];
    let avgs = [28.0, 40.0, 25.0, 15.0, 0.0, 25.0];
    let mut ids = Vec::new();
    for i in 0..FEED_GROUP_NAMES.len() {
        let row = sqlx::query_as::<_, (i32,)>(
			"INSERT INTO feed_groups (name, min_milk_yield, max_milk_yield, avg_milk_yield, avg_weight, number_of_cows) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
		)
		.bind(FEED_GROUP_NAMES[i])
		.bind(mins[i])
		.bind(maxs[i])
		.bind(avgs[i])
		.bind(550.0 + i as f64 * 30.0)
		.bind(40 + (i as i32) * 5)
		.fetch_one(pool)
		.await
		.unwrap();
        ids.push(row.0);
    }
    println!("  feed_groups: {} rows", ids.len());
    ids
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_animals(pool: &PgPool, config: &SeedConfig) -> Vec<AnimalInfo> {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();
    let _start_date = today - Duration::days(config.num_years * 365);
    let mut animals = Vec::new();
    let mut batch = Vec::new();
    let mut used_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    let total_cows = config.num_cows;
    let num_heifers = (total_cows as f64 * 0.2) as usize;
    let num_calves = (total_cows as f64 * 0.1) as usize;
    let num_bulls = (total_cows as f64 * 0.05) as usize;
    let total = total_cows + num_heifers + num_calves + num_bulls;

    let mut unique_name = |rng: &mut rand::rngs::SmallRng, pool: &[&str]| -> String {
        for _ in 0..50 {
            let name = pick(rng, pool).to_string();
            if !used_names.contains(&name) {
                used_names.insert(name.clone());
                return name;
            }
        }
        let base = pick(rng, pool).to_string();
        let n = used_names.iter().filter(|n| n.starts_with(&base)).count();
        let name = format!("{} {}", base, n + 1);
        used_names.insert(name.clone());
        name
    };

    for i in 0..total {
        let (gender, name_pool, age_min, age_max, active_weight): (&str, &[&str], f64, f64, f64) =
            if i < total_cows {
                ("female", COW_NAMES as &[&str], 2.0, 8.0, 0.85)
            } else if i < total_cows + num_heifers {
                ("female", HEIFER_NAMES as &[&str], 0.5, 2.0, 0.95)
            } else if i < total_cows + num_heifers + num_calves {
                ("female", HEIFER_NAMES as &[&str], 0.1, 0.5, 0.97)
            } else {
                ("male", BULL_NAMES as &[&str], 1.0, 5.0, 0.7)
            };

        let age_days = normal_range(
            &mut rng,
            (age_min + age_max) / 2.0 * 365.0,
            180.0,
            age_min * 365.0,
            age_max * 365.0,
        ) as i64;
        let birth_date = today - Duration::days(age_days);
        let active = rng.random_bool(active_weight);
        let name = unique_name(&mut rng, name_pool);
        let life_number = fmt_life_number(&mut rng);
        let hair_color = pick(&mut rng, HAIR_COLORS);
        let ucn = format!("{:012}", rng.random_range(100000000000u64..999999999999u64));
        let loc_idx = rng.random_range(0..LOCATIONS.len());
        let group = rng.random_range(1..7);

        batch.push(format!(
            "('{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, {}, {})",
            life_number,
            name.replace('\'', "''"),
            ucn,
            gender,
            birth_date,
            hair_color,
            LOCATIONS[loc_idx],
            group,
            active,
            rng.random_bool(0.3),
        ));

        animals.push(AnimalInfo {
            id: 0,
            gender: gender.to_string(),
            birth_date,
            active,
            location: Some(LOCATIONS[loc_idx].to_string()),
            group_number: Some(group),
        });

        if batch.len() >= 500 || i == total - 1 {
            let sql = format!(
                "INSERT INTO animals (life_number, name, user_number, gender, birth_date, hair_color_code, location, group_number, active, keep) VALUES {} RETURNING id",
                batch.join(", ")
            );
            let rows: Vec<(i32,)> = sqlx::query_as(&sql).fetch_all(pool).await.unwrap();
            for (idx, (id,)) in rows.iter().enumerate() {
                let base = animals.len() - rows.len() + idx;
                animals[base].id = *id;
            }
            batch.clear();
        }
    }

    println!("  animals: {} rows", animals.len());
    animals
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_bloodlines(pool: &PgPool, animals: &[AnimalInfo]) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let mut batch = Vec::new();
    let mut count = 0;

    for animal in animals {
        let n_breeds = rng.random_range(1..4);
        let picks = pick_n(&mut rng, BLOODLINES, n_breeds);
        let mut remaining = 100.0;
        for (j, code) in picks.iter().enumerate() {
            let pct = if j == picks.len() - 1 {
                remaining
            } else {
                let p = normal_range(
                    &mut rng,
                    remaining / (picks.len() - j) as f64,
                    10.0,
                    5.0,
                    remaining - 5.0 * (picks.len() - j - 1) as f64,
                );
                remaining -= p;
                p
            };
            batch.push(format!("({}, '{}', {})", animal.id, code, pct.round()));
            count += 1;
        }

        if batch.len() >= 1000 {
            let sql = format!(
                "INSERT INTO bloodlines (animal_id, blood_type_code, percentage) VALUES {}",
                batch.join(", ")
            );
            sqlx::query(&sql).execute(pool).await.unwrap();
            batch.clear();
        }
    }

    if !batch.is_empty() {
        let sql = format!(
            "INSERT INTO bloodlines (animal_id, blood_type_code, percentage) VALUES {}",
            batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    println!("  bloodlines: {} rows", count);
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_calvings_and_reproduction(
    pool: &PgPool,
    animals: &[AnimalInfo],
    sire_codes: &[String],
    config: &SeedConfig,
) -> Vec<LactationInfo> {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();
    let start_date = today - Duration::days(config.num_years * 365);

    let females: Vec<&AnimalInfo> = animals.iter().filter(|a| a.gender == "female").collect();
    let mut lactations = Vec::new();

    let mut calving_batch = Vec::new();
    let mut calf_batch = Vec::new();
    let mut ins_batch = Vec::new();
    let mut preg_batch = Vec::new();
    let mut heat_batch = Vec::new();
    let mut dry_batch = Vec::new();

    let mut total_calvings = 0u64;
    let mut total_calves = 0u64;
    let mut total_inseminations = 0u64;
    let mut total_pregnancies = 0u64;
    let mut total_heats = 0u64;
    let mut total_dryoffs = 0u64;

    for female in &females {
        let age_at_start = (start_date - female.birth_date).num_days();
        if age_at_start < 700 {
            continue;
        }

        let mut calving_dates: Vec<NaiveDate> = Vec::new();
        let first_calving_age = normal_range(&mut rng, 850.0, 60.0, 750.0, 1000.0) as i64;
        let mut calving_date = female.birth_date + Duration::days(first_calving_age);

        if calving_date < start_date - Duration::days(400) {
            calving_date = start_date - Duration::days(rng.random_range(0..400));
        }

        let mut lac = 0;
        loop {
            if calving_date > today - Duration::days(30) {
                break;
            }
            lac += 1;
            calving_dates.push(calving_date);

            let _calving_id_offset = total_calvings + calving_dates.len() as u64;

            let num_calves = if rng.random_bool(0.95) { 1 } else { 2 };
            for _ in 0..num_calves {
                let calf_gender = if rng.random_bool(0.5) {
                    "male"
                } else {
                    "female"
                };
                let calf_ln = fmt_life_number(&mut rng);
                let birth_remark = if rng.random_bool(0.9) {
                    "normal"
                } else {
                    pick(
                        &mut rng,
                        &["abnormal_calf", "alive_premature_born", "abortion"],
                    )
                };
                let weight = normal_range(&mut rng, 38.0, 5.0, 25.0, 55.0);
                let born_dead = rng.random_bool(0.03);
                let keep = if born_dead {
                    false
                } else {
                    rng.random_bool(0.85)
                };
                let calf_name = if calf_gender == "female" && !born_dead && rng.random_bool(0.7) {
                    format!("'{}'", pick(&mut rng, HEIFER_NAMES).replace('\'', "''"))
                } else {
                    "NULL".to_string()
                };
                let calf_gender_sql = if born_dead && *birth_remark == *"abortion" {
                    "'male'".to_string()
                } else {
                    format!("'{}'", calf_gender)
                };
                calf_batch.push(format!(
					"((SELECT id FROM calvings WHERE animal_id = {} AND calving_date = '{}' ORDER BY id DESC LIMIT 1), '{}', {}, '{}', {}, {}, {}, {})",
					female.id,
					calving_date,
					calf_ln,
					calf_gender_sql,
					birth_remark,
					keep,
					weight.round(),
					born_dead,
					calf_name,
				));
                total_calves += 1;
            }

            let num_heats = rng.random_range(2..5);
            let first_heat_day = normal_range(&mut rng, 35.0, 10.0, 20.0, 60.0) as i64;
            for h in 0..num_heats {
                let heat_day = first_heat_day + h * 21;
                let heat_date = calving_date + Duration::days(heat_day);
                if heat_date > today {
                    break;
                }
                heat_batch.push(format!("({}, '{}')", female.id, heat_date));
                total_heats += 1;
            }

            let num_ai = rng.random_range(1..4);
            let first_ai_day = first_heat_day + rng.random_range(0..21);
            let mut got_pregnant = false;
            let mut pregnant_at_day = 0i64;

            for ai in 0..num_ai {
                let ai_day = first_ai_day + ai * (21 + rng.random_range(0..7));
                let ai_date = calving_date + Duration::days(ai_day);
                if ai_date > today - Duration::days(30) {
                    break;
                }
                let sire = pick(&mut rng, sire_codes);
                let ins_type = pick(&mut rng, INSEMINATION_TYPES);
                let charge = format!("CH{:06}", rng.random_range(1..999999u32));
                ins_batch.push(format!(
                    "({}, '{}', '{}', '{}', '{}')",
                    female.id, ai_date, sire, ins_type, charge,
                ));
                total_inseminations += 1;

                if !got_pregnant {
                    let conception_chance = if ai == 0 {
                        0.45
                    } else if ai == 1 {
                        0.35
                    } else {
                        0.25
                    };
                    if rng.random_bool(conception_chance) {
                        got_pregnant = true;
                        pregnant_at_day = ai_day;
                        let preg_date = ai_date
                            + Duration::days(normal_range(&mut rng, 30.0, 5.0, 25.0, 40.0) as i64);
                        if preg_date <= today {
                            let preg_type = pick(&mut rng, PREGNANCY_TYPES);
                            preg_batch.push(format!(
                                "({}, '{}', '{}', '{}')",
                                female.id, preg_date, preg_type, ai_date,
                            ));
                            total_pregnancies += 1;
                        }
                    }
                }
            }

            if got_pregnant && pregnant_at_day > 0 {
                let next_calving = calving_date + Duration::days(pregnant_at_day + 283);
                let dry_off = next_calving - Duration::days(60);

                if dry_off <= today && next_calving > today - Duration::days(30) {
                    dry_batch.push(format!("({}, '{}')", female.id, dry_off));
                    total_dryoffs += 1;

                    if calving_dates.len() < 6 && next_calving <= today + Duration::days(60) {
                        let dry_off_opt = Some(dry_off);
                        lactations.push(LactationInfo {
                            animal_id: female.id,
                            calving_date,
                            lac_number: lac,
                            dry_off_date: dry_off_opt,
                            next_calving_date: Some(next_calving),
                        });
                        calving_date = next_calving;
                        continue;
                    }
                }

                lactations.push(LactationInfo {
                    animal_id: female.id,
                    calving_date,
                    lac_number: lac,
                    dry_off_date: if dry_off <= today {
                        Some(dry_off)
                    } else {
                        None
                    },
                    next_calving_date: Some(next_calving),
                });
                if calving_dates.len() < 6 {
                    calving_date = next_calving;
                    continue;
                }
            } else {
                lactations.push(LactationInfo {
                    animal_id: female.id,
                    calving_date,
                    lac_number: lac,
                    dry_off_date: None,
                    next_calving_date: None,
                });
            }

            let next_interval = normal_range(&mut rng, 390.0, 40.0, 340.0, 480.0) as i64;
            calving_date += Duration::days(next_interval);
        }

        for cd in &calving_dates {
            let remarks = if rng.random_bool(0.85) {
                "NULL".to_string()
            } else {
                format!(
                    "'{}'",
                    pick(
                        &mut rng,
                        &["лёгкие", "нормальные", "тяжёлые", "с ветеринарной помощью"]
                    )
                )
            };
            calving_batch.push(format!(
                "({}, '{}', {}, {})",
                female.id,
                cd,
                calving_dates.iter().position(|d| d == cd).unwrap() + 1,
                remarks,
            ));
        }
        total_calvings += calving_dates.len() as u64;

        if calving_batch.len() >= 500 {
            let sql = format!(
                "INSERT INTO calvings (animal_id, calving_date, lac_number, remarks) VALUES {}",
                calving_batch.join(", ")
            );
            sqlx::query(&sql).execute(pool).await.unwrap();
            calving_batch.clear();
        }
    }

    if !calving_batch.is_empty() {
        let sql = format!(
            "INSERT INTO calvings (animal_id, calving_date, lac_number, remarks) VALUES {}",
            calving_batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }

    if !heat_batch.is_empty() {
        for chunk in heat_batch.chunks(5000) {
            let sql = format!(
                "INSERT INTO heats (animal_id, heat_date) VALUES {}",
                chunk.join(", ")
            );
            sqlx::query(&sql).execute(pool).await.unwrap();
        }
    }

    if !ins_batch.is_empty() {
        for chunk in ins_batch.chunks(5000) {
            let sql = format!(
                "INSERT INTO inseminations (animal_id, insemination_date, sire_code, insemination_type, charge_number) VALUES {}",
                chunk.join(", ")
            );
            sqlx::query(&sql).execute(pool).await.unwrap();
        }
    }

    if !preg_batch.is_empty() {
        for chunk in preg_batch.chunks(5000) {
            let sql = format!(
                "INSERT INTO pregnancies (animal_id, pregnancy_date, pregnancy_type, insemination_date) VALUES {}",
                chunk.join(", ")
            );
            sqlx::query(&sql).execute(pool).await.unwrap();
        }
    }

    if !dry_batch.is_empty() {
        for chunk in dry_batch.chunks(5000) {
            let sql = format!(
                "INSERT INTO dry_offs (animal_id, dry_off_date) VALUES {}",
                chunk.join(", ")
            );
            sqlx::query(&sql).execute(pool).await.unwrap();
        }
    }

    if !calf_batch.is_empty() {
        for chunk in calf_batch.chunks(1000) {
            let sql = format!(
                "INSERT INTO calves (calving_id, life_number, gender, birth_remark, keep, weight, born_dead, calf_name) VALUES {}",
                chunk.join(", ")
            );
            sqlx::query(&sql).execute(pool).await.unwrap();
        }
    }

    println!("  calvings: {} rows", total_calvings);
    println!("  calves: {} rows", total_calves);
    println!("  heats: {} rows", total_heats);
    println!("  inseminations: {} rows", total_inseminations);
    println!("  pregnancies: {} rows", total_pregnancies);
    println!("  dry_offs: {} rows", total_dryoffs);

    lactations
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_milk(pool: &PgPool, lactations: &[LactationInfo], config: &SeedConfig) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();
    let start_date = today - Duration::days(config.num_years * 365);

    let mut day_batch = Vec::new();
    let mut visit_batch = Vec::new();
    let mut quality_batch = Vec::new();
    let mut visit_keys = std::collections::HashSet::new();
    let mut total_days = 0u64;
    let mut total_visits = 0u64;
    let mut total_quality = 0u64;

    for lact in lactations {
        let lac_start = lact.calving_date;
        let lac_end = match (lact.dry_off_date, lact.next_calving_date) {
            (Some(dry), _) => dry,
            (None, Some(next)) => next - Duration::days(60),
            _ => today.min(lac_start + Duration::days(350)),
        };

        let effective_start = lac_start.max(start_date);
        let effective_end = lac_end.min(today);

        if effective_start >= effective_end {
            continue;
        }

        let mut date = effective_start;
        while date <= effective_end {
            let days_in_lac = (date - lac_start).num_days();
            let milk = wood_milk(days_in_lac, lact.lac_number, &mut rng);

            if milk > 0.5 {
                day_batch.push(format!(
                    "({}, '{}', {}, {}, {}, {})",
                    lact.animal_id,
                    date,
                    (milk * 10.0).round() / 10.0,
                    (milk * 1000.0 / 360.0).round() / 100.0,
                    normal_range(&mut rng, 8.5, 1.0, 5.0, 15.0).round() / 10.0,
                    normal_range(&mut rng, 0.8, 0.2, 0.3, 1.5).round() / 10.0,
                ));
                total_days += 1;

                let num_visits = rng.random_range(2..4);
                for v in 0..num_visits {
                    let hour = (5 + v * 8 + rng.random_range(0..3)).min(23);
                    let minute = rng.random_range(0..60);
                    let visit_key =
                        format!("{}:{}:{:02}:{:02}", lact.animal_id, date, hour, minute);
                    if !visit_keys.insert(visit_key) {
                        continue;
                    }
                    let visit_milk =
                        (milk / num_visits as f64) * normal_range(&mut rng, 1.0, 0.15, 0.5, 1.5);
                    let duration =
                        (visit_milk * 60.0 / normal_range(&mut rng, 2.0, 0.5, 1.0, 4.0)) as i32;
                    visit_batch.push(format!(
                        "({}, '{} {:02}:{:02}:00+03', {}, {}, {})",
                        lact.animal_id,
                        date,
                        hour,
                        minute,
                        (visit_milk * 10.0).round() / 10.0,
                        duration.max(60),
                        rng.random_range(1..3),
                    ));
                    total_visits += 1;
                }

                let day_of_year = date.ordinal();
                if day_of_year % 7 == 0 {
                    let base_scc = 120000.0 + (lact.lac_number as f64 - 1.0) * 15000.0;
                    let scc = (normal_range(&mut rng, base_scc, 40000.0, 30000.0, 500000.0)
                        / 1000.0)
                        .round() as i32
                        * 1000;
                    let fat = normal_range(&mut rng, 3.8, 0.3, 3.0, 5.0);
                    let protein = normal_range(&mut rng, 3.2, 0.2, 2.6, 4.0);
                    let lactose_pct = normal_range(&mut rng, 4.6, 0.15, 4.0, 5.2);
                    let milkings = rng.random_range(2..3);
                    let refusals = if rng.random_bool(0.05) {
                        rng.random_range(1..2)
                    } else {
                        0
                    };
                    quality_batch.push(format!(
                        "({}, '{}', {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                        lact.animal_id,
                        date,
                        (milk * 10.0).round() / 10.0,
                        (milk * 1000.0 / 360.0).round() / 100.0,
                        normal_range(&mut rng, 8.5, 1.0, 5.0, 15.0).round() / 10.0,
                        normal_range(&mut rng, 0.8, 0.2, 0.3, 1.5).round() / 10.0,
                        (fat * 100.0).round() / 100.0,
                        (protein * 100.0).round() / 100.0,
                        (lactose_pct * 100.0).round() / 100.0,
                        scc,
                        milkings,
                        refusals,
                    ));
                    total_quality += 1;
                }
            }

            date += Duration::days(1);

            if day_batch.len() >= 5000 {
                let sql = format!(
                    "INSERT INTO milk_day_productions (animal_id, date, milk_amount, avg_amount, avg_weight, isk) VALUES {} ON CONFLICT (animal_id, date) DO NOTHING",
                    day_batch.join(", ")
                );
                sqlx::query(&sql).execute(pool).await.unwrap();
                day_batch.clear();
            }
            if visit_batch.len() >= 5000 {
                let sql = format!(
                    "INSERT INTO milk_visits (animal_id, visit_datetime, milk_amount, duration_seconds, milk_destination) VALUES {} ON CONFLICT (animal_id, visit_datetime) DO NOTHING",
                    visit_batch.join(", ")
                );
                sqlx::query(&sql).execute(pool).await.unwrap();
                visit_batch.clear();
            }
            if quality_batch.len() >= 5000 {
                let sql = format!(
                    "INSERT INTO milk_quality (animal_id, date, milk_amount, avg_amount, avg_weight, isk, fat_percentage, protein_percentage, lactose_percentage, scc, milkings, refusals) VALUES {} ON CONFLICT (animal_id, date) DO NOTHING",
                    quality_batch.join(", ")
                );
                sqlx::query(&sql).execute(pool).await.unwrap();
                quality_batch.clear();
            }
        }
    }

    if !day_batch.is_empty() {
        let sql = format!(
            "INSERT INTO milk_day_productions (animal_id, date, milk_amount, avg_amount, avg_weight, isk) VALUES {} ON CONFLICT (animal_id, date) DO NOTHING",
            day_batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    if !visit_batch.is_empty() {
        let sql = format!(
            "INSERT INTO milk_visits (animal_id, visit_datetime, milk_amount, duration_seconds, milk_destination) VALUES {} ON CONFLICT (animal_id, visit_datetime) DO NOTHING",
            visit_batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    if !quality_batch.is_empty() {
        let sql = format!(
            "INSERT INTO milk_quality (animal_id, date, milk_amount, avg_amount, avg_weight, isk, fat_percentage, protein_percentage, lactose_percentage, scc, milkings, refusals) VALUES {} ON CONFLICT (animal_id, date) DO NOTHING",
            quality_batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }

    println!("  milk_day_productions: {} rows", total_days);
    println!("  milk_visits: {} rows", total_visits);
    println!("  milk_quality: {} rows", total_quality);
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_feed(pool: &PgPool, lactations: &[LactationInfo], config: &SeedConfig) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();
    let start_date = today - Duration::days(config.num_years * 365);

    let mut day_batch = Vec::new();
    let mut visit_batch = Vec::new();
    let mut visit_keys = std::collections::HashSet::new();
    let mut total_days = 0u64;
    let mut total_visits = 0u64;

    for lact in lactations {
        let lac_start = lact.calving_date;
        let lac_end = match (lact.dry_off_date, lact.next_calving_date) {
            (Some(dry), _) => dry,
            (None, Some(next)) => next - Duration::days(60),
            _ => today.min(lac_start + Duration::days(350)),
        };
        let effective_start = lac_start.max(start_date);
        let effective_end = lac_end.min(today);
        if effective_start >= effective_end {
            continue;
        }

        let mut date = effective_start;
        while date <= effective_end {
            let days_in_lac = (date - lac_start).num_days();
            let milk_est = wood_milk(days_in_lac, lact.lac_number, &mut rng);
            let feed_total = (milk_est * normal_range(&mut rng, 0.38, 0.05, 0.25, 0.55)
                + normal_range(&mut rng, 3.0, 1.0, 1.0, 6.0))
                * 10.0;

            if feed_total > 5.0 {
                let feed_number = rng.random_range(1..4);
                let total_val = (feed_total).round() / 10.0;
                let rest = if rng.random_bool(0.15) {
                    Some(
                        (feed_total * normal_range(&mut rng, 0.08, 0.06, 0.01, 0.3)).round() as i32,
                    )
                } else {
                    None
                };
                day_batch.push(format!(
                    "({}, '{}', {}, {}, {})",
                    lact.animal_id,
                    date,
                    feed_number,
                    total_val,
                    rest.map(|r| r.to_string()).unwrap_or("NULL".to_string()),
                ));
                total_days += 1;

                let num_feed_visits = rng.random_range(3..6);
                let per_visit = feed_total / num_feed_visits as f64;
                for fv in 0..num_feed_visits {
                    let hour = (6 + fv * 4 + rng.random_range(0..3)).min(23);
                    let minute = rng.random_range(0..60);
                    let visit_key =
                        format!("{}:{}:{:02}:{:02}", lact.animal_id, date, hour, minute);
                    if !visit_keys.insert(visit_key) {
                        continue;
                    }
                    let fv_amount = per_visit * normal_range(&mut rng, 1.0, 0.2, 0.3, 2.0);
                    let duration =
                        (fv_amount * 60.0 / normal_range(&mut rng, 2.5, 0.5, 1.0, 5.0)) as i32;
                    visit_batch.push(format!(
                        "({}, '{} {:02}:{:02}:00+03', {}, {}, {})",
                        lact.animal_id,
                        date,
                        hour,
                        minute,
                        rng.random_range(1..4),
                        (fv_amount).round() / 10.0,
                        duration.max(30),
                    ));
                    total_visits += 1;
                }
            }

            date += Duration::days(1);

            if day_batch.len() >= 5000 {
                let sql = format!(
                    "INSERT INTO feed_day_amounts (animal_id, feed_date, feed_number, total, rest_feed) VALUES {} ON CONFLICT (animal_id, feed_date, feed_number) DO NOTHING",
                    day_batch.join(", ")
                );
                sqlx::query(&sql).execute(pool).await.unwrap();
                day_batch.clear();
            }
            if visit_batch.len() >= 5000 {
                let sql = format!(
                    "INSERT INTO feed_visits (animal_id, visit_datetime, feed_number, amount, duration_seconds) VALUES {} ON CONFLICT (animal_id, visit_datetime) DO NOTHING",
                    visit_batch.join(", ")
                );
                sqlx::query(&sql).execute(pool).await.unwrap();
                visit_batch.clear();
            }
        }
    }

    if !day_batch.is_empty() {
        let sql = format!(
            "INSERT INTO feed_day_amounts (animal_id, feed_date, feed_number, total, rest_feed) VALUES {} ON CONFLICT (animal_id, feed_date, feed_number) DO NOTHING",
            day_batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    if !visit_batch.is_empty() {
        let sql = format!(
            "INSERT INTO feed_visits (animal_id, visit_datetime, feed_number, amount, duration_seconds) VALUES {} ON CONFLICT (animal_id, visit_datetime) DO NOTHING",
            visit_batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }

    println!("  feed_day_amounts: {} rows", total_days);
    println!("  feed_visits: {} rows", total_visits);
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_fitness(pool: &PgPool, lactations: &[LactationInfo], config: &SeedConfig) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();
    let start_date = today - Duration::days(config.num_years * 365);

    let mut act_batch = Vec::new();
    let mut rum_batch = Vec::new();
    let mut total_activities = 0u64;
    let mut total_ruminations = 0u64;

    for lact in lactations {
        let lac_start = lact.calving_date;
        let lac_end = match (lact.dry_off_date, lact.next_calving_date) {
            (Some(dry), _) => dry,
            (None, Some(next)) => next - Duration::days(60),
            _ => today.min(lac_start + Duration::days(350)),
        };
        let effective_start = lac_start.max(start_date);
        let effective_end = lac_end.min(today);
        if effective_start >= effective_end {
            continue;
        }

        let mut date = effective_start;
        while date <= effective_end {
            let base_activity: f64 = normal_range(&mut rng, 350.0, 80.0, 200.0, 600.0);
            let num_readings = rng.random_range(1..4);
            for _ in 0..num_readings {
                let hour = rng.random_range(0..24);
                let minute = rng.random_range(0..60);
                let act_val =
                    (base_activity * normal_range(&mut rng, 1.0, 0.15, 0.5, 1.8)).round() as i32;
                let heat = act_val > 500;
                act_batch.push(format!(
                    "({}, '{} {:02}:{:02}:00+03', {}, {})",
                    lact.animal_id, date, hour, minute, act_val, heat,
                ));
                total_activities += 1;
            }

            let rumination = normal_range(&mut rng, 480.0, 80.0, 200.0, 700.0).round() as i32;
            let eating = normal_range(&mut rng, 240.0, 40.0, 120.0, 400.0).round() as i32;
            rum_batch.push(format!(
                "({}, '{}', {}, {})",
                lact.animal_id, date, eating, rumination,
            ));
            total_ruminations += 1;

            date += Duration::days(1);

            if act_batch.len() >= 5000 {
                let sql = format!(
                    "INSERT INTO activities (animal_id, activity_datetime, activity_counter, heat_attention) VALUES {} ON CONFLICT (animal_id, activity_datetime) DO NOTHING",
                    act_batch.join(", ")
                );
                sqlx::query(&sql).execute(pool).await.unwrap();
                act_batch.clear();
            }
            if rum_batch.len() >= 5000 {
                let sql = format!(
                    "INSERT INTO ruminations (animal_id, date, eating_seconds, rumination_minutes) VALUES {} ON CONFLICT (animal_id, date) DO NOTHING",
                    rum_batch.join(", ")
                );
                sqlx::query(&sql).execute(pool).await.unwrap();
                rum_batch.clear();
            }
        }
    }

    if !act_batch.is_empty() {
        let sql = format!(
            "INSERT INTO activities (animal_id, activity_datetime, activity_counter, heat_attention) VALUES {} ON CONFLICT (animal_id, activity_datetime) DO NOTHING",
            act_batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    if !rum_batch.is_empty() {
        let sql = format!(
            "INSERT INTO ruminations (animal_id, date, eating_seconds, rumination_minutes) VALUES {} ON CONFLICT (animal_id, date) DO NOTHING",
            rum_batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }

    println!("  activities: {} rows", total_activities);
    println!("  ruminations: {} rows", total_ruminations);
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_bulk_tank(pool: &PgPool, config: &SeedConfig) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();
    let start_date = today - Duration::days(config.num_years * 365);

    let mut batch = Vec::new();
    let mut count = 0u64;

    let mut date = start_date;
    while date <= today {
        let fat = normal_range(&mut rng, 3.85, 0.25, 3.2, 4.5);
        let protein = normal_range(&mut rng, 3.25, 0.2, 2.7, 3.9);
        let lactose = normal_range(&mut rng, 4.6, 0.12, 4.2, 5.0);
        let scc = (normal_range(&mut rng, 180000.0, 40000.0, 80000.0, 350000.0) / 1000.0).round()
            as i32
            * 1000;
        let ffa = normal_range(&mut rng, 0.8, 0.3, 0.2, 2.0);

        batch.push(format!(
            "('{}', {}, {}, {}, {}, {})",
            date,
            (fat * 100.0).round() / 100.0,
            (protein * 100.0).round() / 100.0,
            (lactose * 100.0).round() / 100.0,
            scc,
            (ffa * 100.0).round() / 100.0,
        ));
        count += 1;
        date += Duration::days(7);
    }

    for chunk in batch.chunks(5000) {
        let sql = format!(
            "INSERT INTO bulk_tank_tests (date, fat, protein, lactose, scc, ffa) VALUES {}",
            chunk.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    println!("  bulk_tank_tests: {} rows", count);
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_grazing(pool: &PgPool, config: &SeedConfig) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();
    let start_date = today - Duration::days(config.num_years * 365);

    let mut batch = Vec::new();
    let mut count = 0u64;

    let mut date = start_date;
    while date <= today {
        let month = date.month();
        let is_grazing_season = matches!(month, 5..=10);
        if is_grazing_season {
            let animal_count = rng.random_range(50..200);
            let pasture_time = rng.random_range(180..600);
            let lac_period = match month {
                5..=6 => "ранняя лактация",
                7..=8 => "средняя лактация",
                _ => "поздняя лактация",
            };
            batch.push(format!(
                "('{}', {}, {}, '{}')",
                date, animal_count, pasture_time, lac_period,
            ));
            count += 1;
        }
        date += Duration::days(1);
    }

    for chunk in batch.chunks(5000) {
        let sql = format!(
            "INSERT INTO grazing_data (date, animal_count, pasture_time, lactation_period) VALUES {} ON CONFLICT (date) DO NOTHING",
            chunk.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    println!("  grazing_data: {} rows", count);
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_transfers(pool: &PgPool, animals: &[AnimalInfo]) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();

    let mut batch = Vec::new();
    let mut count = 0u64;

    for animal in animals {
        if !animal.active {
            let transfer_date = animal.birth_date + Duration::days(rng.random_range(365..1500));
            if transfer_date <= today {
                let reason = pick(&mut rng, TRANSFER_REASONS);
                batch.push(format!(
                    "({}, '{} {:02}:{:02}:00+03', '{}', '{}', '{}')",
                    animal.id,
                    transfer_date,
                    rng.random_range(8..18),
                    rng.random_range(0..60),
                    "выбытие",
                    animal
                        .location
                        .as_deref()
                        .unwrap_or("Коровник 1 (основной)"),
                    reason,
                ));
                count += 1;
            }
        }

        if rng.random_bool(0.3) {
            let transfer_date = animal.birth_date + Duration::days(rng.random_range(100..600));
            if transfer_date <= today {
                let _reason = pick(&mut rng, &["Перевод в группу", "На пастбище", "С пастбища"]);
                let from_loc = animal
                    .location
                    .as_deref()
                    .unwrap_or("Коровник 1 (основной)");
                let to_loc = pick(&mut rng, LOCATIONS);
                batch.push(format!(
                    "({}, '{} {:02}:{:02}:00+03', '{}', '{}', '{}')",
                    animal.id,
                    transfer_date,
                    rng.random_range(8..18),
                    rng.random_range(0..60),
                    "перемещение",
                    from_loc,
                    to_loc,
                ));
                count += 1;
            }
        }
    }

    for chunk in batch.chunks(1000) {
        let sql = format!(
            "INSERT INTO transfers (animal_id, transfer_date, transfer_type, from_location, to_location) VALUES {}",
            chunk.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    println!("  transfers: {} rows", count);
}

pub async fn seed_sync_log(pool: &PgPool) {
    let entities = [
        "animals",
        "calvings",
        "inseminations",
        "milk_productions",
        "feed_data",
        "health_data",
        "grazing",
        "bulk_tank",
    ];
    let mut batch = Vec::new();
    for entity in &entities {
        batch.push(format!(
            "('{}', '{}', 'completed', {})",
            entity,
            chrono::Utc::now(),
            100 + GLOBAL_RNG.lock().unwrap().random_range(0..1000)
        ));
    }
    let sql = format!(
        "INSERT INTO sync_log (entity_type, last_synced_at, status, records_synced) VALUES {}",
        batch.join(", ")
    );
    sqlx::query(&sql).execute(pool).await.unwrap();
    println!("  sync_log: {} rows", entities.len());
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_milk_visit_quality(
    pool: &PgPool,
    lactations: &[LactationInfo],
    config: &SeedConfig,
) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();
    let start_date = today - Duration::days(config.num_years * 365);

    let mut batch = Vec::new();
    let mut count = 0u64;

    for lact in lactations {
        let lac_start = lact.calving_date;
        let lac_end = match (lact.dry_off_date, lact.next_calving_date) {
            (Some(dry), _) => dry,
            (None, Some(next)) => next - Duration::days(60),
            _ => today.min(lac_start + Duration::days(350)),
        };
        let effective_start = lac_start.max(start_date);
        let effective_end = lac_end.min(today);
        if effective_start >= effective_end {
            continue;
        }

        let mut date = effective_start;
        while date <= effective_end {
            let num_visits = rng.random_range(1..4);
            for v in 0..num_visits {
                let hour = 5 + v * 8 + rng.random_range(0..3);
                let minute = rng.random_range(0..60);
                let visit_dt = format!("{} {:02}:{:02}:00+03", date, hour.min(23), minute);

                let milk_yield: f64 = normal_range(&mut rng, 12.0, 4.0, 3.0, 25.0);
                let success = rng.random_bool(0.95);
                let device = rng.random_range(1..=3);

                let base_cond = normal_range(&mut rng, 65.0, 8.0, 50.0, 85.0);
                let lf_cond = base_cond.round() as i32;
                let lr_cond =
                    (base_cond + normal_range(&mut rng, 0.0, 3.0, -5.0, 5.0)).round() as i32;
                let rf_cond =
                    (base_cond + normal_range(&mut rng, 0.0, 3.0, -5.0, 5.0)).round() as i32;
                let rr_cond =
                    (base_cond + normal_range(&mut rng, 0.0, 3.0, -5.0, 5.0)).round() as i32;

                let lf_colour = if rng.random_bool(0.05) {
                    Some("W")
                } else {
                    None
                };
                let lr_colour = if rng.random_bool(0.04) {
                    Some("W")
                } else {
                    None
                };
                let rf_colour = if rng.random_bool(0.04) {
                    Some("Y")
                } else {
                    None
                };
                let rr_colour = if rng.random_bool(0.03) {
                    Some("W")
                } else {
                    None
                };

                let weight = (normal_range(&mut rng, 600.0, 80.0, 400.0, 800.0)).round() as i32;
                let temp = normal_range(&mut rng, 37.5, 0.3, 36.5, 39.0);

                batch.push(format!(
                    "({}, '{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                    lact.animal_id,
                    visit_dt,
                    format!("{} {:02}:{:02}:00+03", date, hour.min(23), minute),
                    device,
                    success,
                    (milk_yield * 100.0).round() / 100.0,
                    rng.random_range(1..=4),
                    (temp * 10.0).round() / 10.0,
                    weight,
                    lf_colour.map_or("NULL".to_string(), |c| format!("'{}'", c)),
                    lr_colour.map_or("NULL".to_string(), |c| format!("'{}'", c)),
                    rf_colour.map_or("NULL".to_string(), |c| format!("'{}'", c)),
                    rr_colour.map_or("NULL".to_string(), |c| format!("'{}'", c)),
                    lf_cond,
                    lr_cond,
                    rf_cond,
                    rr_cond,
                ));
                count += 1;
            }

            date += Duration::days(1);

            if batch.len() >= 5000 {
                let sql = format!(
                    "INSERT INTO milk_visit_quality (animal_id, visit_datetime, milking_start_date, device_address, success_milking, milk_yield, bottle_number, milk_temperature, weight, lf_colour_code, lr_colour_code, rf_colour_code, rr_colour_code, lf_conductivity, lr_conductivity, rf_conductivity, rr_conductivity) VALUES {} ON CONFLICT (animal_id, visit_datetime) DO NOTHING",
                    batch.join(", ")
                );
                sqlx::query(&sql).execute(pool).await.unwrap();
                batch.clear();
            }
        }
    }

    if !batch.is_empty() {
        let sql = format!(
            "INSERT INTO milk_visit_quality (animal_id, visit_datetime, milking_start_date, device_address, success_milking, milk_yield, bottle_number, milk_temperature, weight, lf_colour_code, lr_colour_code, rf_colour_code, rr_colour_code, lf_conductivity, lr_conductivity, rf_conductivity, rr_conductivity) VALUES {} ON CONFLICT (animal_id, visit_datetime) DO NOTHING",
            batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    println!("  milk_visit_quality: {} rows", count);
}

#[allow(clippy::await_holding_lock)]
pub async fn seed_robot_milk_data(
    pool: &PgPool,
    lactations: &[LactationInfo],
    config: &SeedConfig,
) {
    let mut rng = GLOBAL_RNG.lock().unwrap();
    let today = chrono::Utc::now().date_naive();
    let start_date = today - Duration::days(config.num_years * 365);

    let mut batch = Vec::new();
    let mut count = 0u64;

    for lact in lactations {
        let lac_start = lact.calving_date;
        let lac_end = match (lact.dry_off_date, lact.next_calving_date) {
            (Some(dry), _) => dry,
            (None, Some(next)) => next - Duration::days(60),
            _ => today.min(lac_start + Duration::days(350)),
        };
        let effective_start = lac_start.max(start_date);
        let effective_end = lac_end.min(today);
        if effective_start >= effective_end {
            continue;
        }

        let mut date = effective_start;
        while date <= effective_end {
            let num_milkings = rng.random_range(1..4);
            for _ in 0..num_milkings {
                let device = rng.random_range(1..=3);
                let milk_speed = normal_range(&mut rng, 2.0, 0.5, 0.8, 4.0);
                let milk_speed_max = milk_speed * normal_range(&mut rng, 1.2, 0.1, 1.0, 1.5);

                let base_time = (120.0 / milk_speed) as i32;
                let lf_time =
                    (base_time as f64 * normal_range(&mut rng, 1.0, 0.1, 0.7, 1.3)).round() as i32;
                let lr_time =
                    (base_time as f64 * normal_range(&mut rng, 1.0, 0.1, 0.7, 1.3)).round() as i32;
                let rf_time =
                    (base_time as f64 * normal_range(&mut rng, 1.0, 0.1, 0.7, 1.3)).round() as i32;
                let rr_time =
                    (base_time as f64 * normal_range(&mut rng, 1.0, 0.1, 0.7, 1.3)).round() as i32;

                let dead_base = normal_range(&mut rng, 15.0, 5.0, 5.0, 30.0).round() as i32;

                batch.push(format!(
                    "({}, '{} {:02}:{:02}:00+03', {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                    lact.animal_id,
                    date,
                    rng.random_range(5..22),
                    rng.random_range(0..60),
                    device,
                    (milk_speed * 100.0).round() / 100.0,
                    (milk_speed_max * 100.0).round() / 100.0,
                    lf_time, lr_time, rf_time, rr_time,
                    dead_base + rng.random_range(-3..3),
                    dead_base + rng.random_range(-3..3),
                    dead_base + rng.random_range(-3..3),
                    dead_base + rng.random_range(-3..3),
                    rng.random_range(80..120),
                    rng.random_range(-5..5),
                    rng.random_range(60..90),
                    rng.random_range(80..120),
                    rng.random_range(-5..5),
                    rng.random_range(60..90),
                    rng.random_range(80..120),
                    rng.random_range(-5..5),
                    rng.random_range(60..90),
                    rng.random_range(80..120),
                    rng.random_range(-5..5),
                    rng.random_range(60..90),
                ));
                count += 1;
            }

            date += Duration::days(1);

            if batch.len() >= 5000 {
                let sql = format!(
                    "INSERT INTO robot_milk_data (animal_id, milking_date, device_address, milk_speed, milk_speed_max, lf_milk_time, lr_milk_time, rf_milk_time, rr_milk_time, lf_dead_milk_time, lr_dead_milk_time, rf_dead_milk_time, rr_dead_milk_time, lf_x_position, lf_y_position, lf_z_position, lr_x_position, lr_y_position, lr_z_position, rf_x_position, rf_y_position, rf_z_position, rr_x_position, rr_y_position, rr_z_position) VALUES {} ON CONFLICT (animal_id, milking_date) DO NOTHING",
                    batch.join(", ")
                );
                sqlx::query(&sql).execute(pool).await.unwrap();
                batch.clear();
            }
        }
    }

    if !batch.is_empty() {
        let sql = format!(
            "INSERT INTO robot_milk_data (animal_id, milking_date, device_address, milk_speed, milk_speed_max, lf_milk_time, lr_milk_time, rf_milk_time, rr_milk_time, lf_dead_milk_time, lr_dead_milk_time, rf_dead_milk_time, rr_dead_milk_time, lf_x_position, lf_y_position, lf_z_position, lr_x_position, lr_y_position, lr_z_position, rf_x_position, rf_y_position, rf_z_position, rr_x_position, rr_y_position, rr_z_position) VALUES {} ON CONFLICT (animal_id, milking_date) DO NOTHING",
            batch.join(", ")
        );
        sqlx::query(&sql).execute(pool).await.unwrap();
    }
    println!("  robot_milk_data: {} rows", count);
}
