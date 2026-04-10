pub mod generators;
pub mod names;

use sqlx::PgPool;
use generators::SeedConfig;

pub async fn truncate_all(pool: &PgPool, keep_admin: bool) {
	println!("Очистка таблиц...");
	let tables = [
		"sync_log", "grazing_data", "bulk_tank_tests",
		"ruminations", "activities",
		"feed_visits", "feed_day_amounts",
		"milk_quality", "milk_visits", "milk_day_productions",
		"dry_offs", "heats", "pregnancies", "inseminations",
		"calves", "calvings",
		"bloodlines", "transfers",
		"feed_groups", "feed_types",
		"sires",
		"animals",
		"contacts", "locations",
	];
	for t in &tables {
		sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", t))
			.execute(pool)
			.await
			.unwrap();
	}
	if !keep_admin {
		sqlx::query("TRUNCATE TABLE users CASCADE")
			.execute(pool)
			.await
			.unwrap();
		sqlx::query(
			"INSERT INTO users (username, password_hash, role) VALUES ($1, $2, 'admin')",
		)
		.bind("admin")
		.bind(bcrypt::hash("admin", bcrypt::DEFAULT_COST).unwrap())
		.execute(pool)
		.await
		.unwrap();
	}
	println!("  Все таблицы очищены.");
}

pub async fn seed_all(pool: &PgPool, config: &SeedConfig) {
	let start = std::time::Instant::now();

	println!("\nГенерация данных (коров: {}, период: {} лет):\n", config.num_cows, config.num_years);

	let _locations = generators::seed_locations(pool).await;
	let _contacts = generators::seed_contacts(pool).await;
	let sire_codes = generators::seed_sires(pool).await;
	let _feed_types = generators::seed_feed_types(pool).await;
	let _feed_groups = generators::seed_feed_groups(pool).await;

	let animals = generators::seed_animals(pool, config).await;
	generators::seed_bloodlines(pool, &animals).await;

	let lactations = generators::seed_calvings_and_reproduction(pool, &animals, &sire_codes, config).await;

	generators::seed_milk(pool, &lactations, config).await;
	generators::seed_feed(pool, &lactations, config).await;
	generators::seed_fitness(pool, &lactations, config).await;

	generators::seed_bulk_tank(pool, config).await;
	generators::seed_grazing(pool, config).await;
	generators::seed_transfers(pool, &animals).await;
	generators::seed_sync_log(pool).await;

	let elapsed = start.elapsed();
	println!("\nГотово за {:.1} секунд.", elapsed.as_secs_f64());
}
