use clap::Parser;

#[derive(Parser)]
#[command(name = "seed", about = "Генерация mock-данных для молочной фермы")]
struct Args {
	#[arg(long, default_value_t = 300)]
	cows: usize,

	#[arg(long, default_value_t = 3)]
	years: i64,

	#[arg(long, default_value_t = false)]
	keep_admin: bool,
}

#[tokio::main]
async fn main() {
	dotenvy::dotenv().ok();
	let args = Args::parse();

	let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
	let pool = sqlx::PgPool::connect(&database_url).await.expect("Failed to connect to DB");

	milk_farm_backend::seed::truncate_all(&pool, args.keep_admin).await;
	milk_farm_backend::seed::seed_all(&pool, &milk_farm_backend::seed::generators::SeedConfig {
		num_cows: args.cows,
		num_years: args.years,
	}).await;

	pool.close().await;
}
