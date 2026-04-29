use criterion::{Criterion, criterion_group, criterion_main};
use sqlx::PgPool;

fn get_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for SQL benchmarks");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(4)
            .connect(&database_url)
            .await
            .expect("Failed to connect to database")
    })
}

fn bench_analytics_kpi(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("analytics_kpi");
    group.sample_size(20);
    group.bench_function("kpi_full", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                milk_farm_backend::services::analytics_service::kpi(&pool)
                    .await
                    .unwrap();
            }
        });
    });
    group.finish();
}

fn bench_analytics_calving_interval(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("analytics_calving_interval");
    group.sample_size(30);
    group.bench_function("avg_calving_interval", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                let row: Option<(Option<f64>,)> = sqlx::query_as(
                    "SELECT AVG(c2.calving_date - c1.calving_date)::float8
                     FROM calvings c1
                     JOIN calvings c2 ON c1.animal_id = c2.animal_id AND c2.calving_date > c1.calving_date
                     WHERE NOT EXISTS (
                         SELECT 1 FROM calvings c3
                         WHERE c3.animal_id = c1.animal_id
                         AND c3.calving_date > c1.calving_date AND c3.calving_date < c2.calving_date
                     )",
                )
                .fetch_optional(&pool)
                .await
                .unwrap();
                row
            }
        });
    });
    group.finish();
}

fn bench_analytics_conception_rate(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("analytics_conception_rate");
    group.sample_size(30);
    group.bench_function("conception_rate", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                let row: Option<(i64, i64)> = sqlx::query_as(
                    "SELECT
                        (SELECT COUNT(*) FROM inseminations WHERE insemination_date >= CURRENT_DATE - INTERVAL '12 months')::int8,
                        (SELECT COUNT(*) FROM pregnancies WHERE pregnancy_date >= CURRENT_DATE - INTERVAL '12 months')::int8",
                )
                .fetch_optional(&pool)
                .await
                .unwrap();
                row
            }
        });
    });
    group.finish();
}

fn bench_analytics_milk_by_lactation(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("analytics_milk_by_lactation");
    group.sample_size(30);
    group.bench_function("milk_by_lactation", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                let rows: Vec<(i32, Option<f64>)> = sqlx::query_as(
                    "SELECT c.lac_number, AVG(m.milk_amount)::float8
                     FROM milk_day_productions m
                     JOIN calvings c ON c.animal_id = m.animal_id
                     WHERE c.lac_number IS NOT NULL
                       AND m.date >= c.calving_date
                       AND m.date < c.calving_date + INTERVAL '400 days'
                     GROUP BY c.lac_number
                     ORDER BY c.lac_number",
                )
                .fetch_all(&pool)
                .await
                .unwrap();
                rows
            }
        });
    });
    group.finish();
}

fn bench_analytics_culling_risk(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("analytics_culling_risk");
    group.sample_size(20);
    group.bench_function("culling_risk_calc", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                let rows: Vec<(i32, Option<String>, Option<String>, Option<f64>, Option<f64>, Option<f64>, Option<i64>)> = sqlx::query_as(
                    "SELECT a.id, a.name, a.life_number,
                            latest_milk.milk as recent_milk,
                            latest_scc.scc as recent_scc,
                            ci.avg_interval as avg_interval,
                            EXTRACT(YEAR FROM AGE(CURRENT_DATE, a.birth_date))::int8 as age_years
                     FROM animals a
                     LEFT JOIN LATERAL (
                         SELECT AVG(m.milk_amount)::float8 as milk
                         FROM milk_day_productions m WHERE m.animal_id = a.id AND m.date >= CURRENT_DATE - INTERVAL '30 days'
                     ) latest_milk ON true
                     LEFT JOIN LATERAL (
                         SELECT AVG(q.scc)::float8 as scc
                         FROM milk_quality q WHERE q.animal_id = a.id AND q.date >= CURRENT_DATE - INTERVAL '90 days'
                     ) latest_scc ON true
                     LEFT JOIN LATERAL (
                          SELECT AVG(c2.calving_date - c1.calving_date)::float8 as avg_interval
                         FROM calvings c1
                         JOIN calvings c2 ON c1.animal_id = c2.animal_id AND c2.calving_date > c1.calving_date
                         WHERE c1.animal_id = a.id
                         AND NOT EXISTS (SELECT 1 FROM calvings c3 WHERE c3.animal_id = c1.animal_id AND c3.calving_date > c1.calving_date AND c3.calving_date < c2.calving_date)
                     ) ci ON true
                     WHERE a.active = true AND a.gender = 'female'
                     ORDER BY a.id",
                )
                .fetch_all(&pool)
                .await
                .unwrap();
                rows.len()
            }
        });
    });
    group.finish();
}

fn bench_reports_milk_summary(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("reports_milk_summary");
    group.sample_size(30);
    group.bench_function("milk_summary_no_filter", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                milk_farm_backend::services::reports_service::milk_summary(&pool, None, None)
                    .await
                    .unwrap();
            }
        });
    });

    group.bench_function("milk_summary_30_days", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            let from = chrono::NaiveDate::from(
                chrono::Local::now().date_naive() - chrono::Duration::days(30),
            );
            async move {
                milk_farm_backend::services::reports_service::milk_summary(&pool, Some(from), None)
                    .await
                    .unwrap();
            }
        });
    });
    group.finish();
}

fn bench_reports_reproduction_summary(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("reports_reproduction_summary");
    group.sample_size(30);
    group.bench_function("reproduction_summary", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                milk_farm_backend::services::reports_service::reproduction_summary(&pool, None, None)
                    .await
                    .unwrap();
            }
        });
    });
    group.finish();
}

fn bench_reports_herd_overview(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("reports_herd_overview");
    group.sample_size(20);
    group.bench_function("herd_overview_no_filter", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                milk_farm_backend::services::reports_service::herd_overview(&pool, None, None)
                    .await
                    .unwrap();
            }
        });
    });

    group.bench_function("herd_overview_7_days", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            let from = chrono::NaiveDate::from(
                chrono::Local::now().date_naive() - chrono::Duration::days(7),
            );
            async move {
                milk_farm_backend::services::reports_service::herd_overview(&pool, Some(from), None)
                    .await
                    .unwrap();
            }
        });
    });
    group.finish();
}

fn bench_reports_feed_summary(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("reports_feed_summary");
    group.sample_size(30);
    group.bench_function("feed_summary", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                milk_farm_backend::services::reports_service::feed_summary(&pool, None, None)
                    .await
                    .unwrap();
            }
        });
    });
    group.finish();
}

fn bench_reports_rest_feed(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("reports_rest_feed");
    group.sample_size(20);
    group.bench_function("rest_feed_report", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                milk_farm_backend::services::reports_service::rest_feed_report(&pool, None, None)
                    .await
                    .unwrap();
            }
        });
    });
    group.finish();
}

fn bench_raw_animal_count(c: &mut Criterion) {
    let pool = get_pool();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("raw_queries");
    group.sample_size(50);
    group.bench_function("count_animals", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                let count: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM animals WHERE active = true")
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                count
            }
        });
    });

    group.bench_function("count_milk_day_productions", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                let count: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM milk_day_productions")
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                count
            }
        });
    });

    group.bench_function("count_calvings", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                let count: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM calvings")
                        .fetch_one(&pool)
                        .await
                        .unwrap();
                count
            }
        });
    });

    group.bench_function("avg_milk_30d", |b| {
        b.to_async(&rt).iter(|| {
            let pool = pool.clone();
            async move {
                let avg: (Option<f64>,) = sqlx::query_as(
                    "SELECT AVG(milk_amount)::float8 FROM milk_day_productions WHERE date >= CURRENT_DATE - INTERVAL '30 days'",
                )
                .fetch_one(&pool)
                .await
                .unwrap();
                avg
            }
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_analytics_kpi,
    bench_analytics_calving_interval,
    bench_analytics_conception_rate,
    bench_analytics_milk_by_lactation,
    bench_analytics_culling_risk,
    bench_reports_milk_summary,
    bench_reports_reproduction_summary,
    bench_reports_herd_overview,
    bench_reports_feed_summary,
    bench_reports_rest_feed,
    bench_raw_animal_count,
);
criterion_main!(benches);
