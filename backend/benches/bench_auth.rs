use criterion::{Criterion, criterion_group, criterion_main};

static JWT_SECRET: &str = "test_secret_key_32_characters_long!";

fn bench_jwt_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_create");

    group.bench_function("create_access_token", |b| {
        b.iter(|| {
            milk_farm_backend::middleware::auth::create_access_token(
                "admin",
                "admin",
                false,
                JWT_SECRET,
                900,
            )
            .unwrap()
        });
    });

    group.bench_function("create_refresh_token", |b| {
        b.iter(|| {
            milk_farm_backend::middleware::auth::create_refresh_token(
                "admin",
                "admin",
                JWT_SECRET,
                604800,
            )
            .unwrap()
        });
    });

    group.bench_function("create_100_tokens", |b| {
        b.iter(|| {
            (0..100)
                .map(|i| {
                    milk_farm_backend::middleware::auth::create_access_token(
                        &format!("user_{i}"),
                        "user",
                        false,
                        JWT_SECRET,
                        900,
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>()
        });
    });
    group.finish();
}

fn bench_jwt_verify(c: &mut Criterion) {
    let token = milk_farm_backend::middleware::auth::create_access_token(
        "admin",
        "admin",
        false,
        JWT_SECRET,
        900,
    )
    .unwrap();

    let mut group = c.benchmark_group("jwt_verify");

    group.bench_function("verify_access_token", |b| {
        b.iter(|| {
            milk_farm_backend::middleware::auth::verify_token(&token, JWT_SECRET).unwrap()
        });
    });

    group.bench_function("verify_100_tokens", |b| {
        b.iter(|| {
            let tokens: Vec<String> = (0..100)
                .map(|i| {
                    milk_farm_backend::middleware::auth::create_access_token(
                        &format!("user_{i}"),
                        "user",
                        false,
                        JWT_SECRET,
                        900,
                    )
                    .unwrap()
                })
                .collect();
            tokens
                .iter()
                .map(|t| {
                    milk_farm_backend::middleware::auth::verify_token(t, JWT_SECRET).unwrap()
                })
                .count()
        });
    });
    group.finish();
}

fn bench_bcrypt(c: &mut Criterion) {
    let password = "secure_password_123!";

    let mut group = c.benchmark_group("bcrypt");
    group.sample_size(10);

    group.bench_function("hash_default_cost", |b| {
        b.iter(|| {
            bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap()
        });
    });

    group.bench_function("verify_correct_password", |b| {
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
        b.iter(|| {
            bcrypt::verify(password, &hash).unwrap()
        });
    });

    group.bench_function("verify_wrong_password", |b| {
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
        b.iter(|| {
            bcrypt::verify("wrong_password", &hash).unwrap()
        });
    });
    group.finish();
}

fn bench_auth_flow_combined(c: &mut Criterion) {
    let mut group = c.benchmark_group("auth_flow");
    group.sample_size(20);

    group.bench_function("full_login_flow", |b| {
        b.iter(|| {
            let password = "test_password_123";
            let hash = bcrypt::hash(password, 4).unwrap();
            let verified = bcrypt::verify(password, &hash).unwrap();
            assert!(verified);
            let token = milk_farm_backend::middleware::auth::create_access_token(
                "admin",
                "admin",
                false,
                JWT_SECRET,
                900,
            )
            .unwrap();
            let claims =
                milk_farm_backend::middleware::auth::verify_token(&token, JWT_SECRET).unwrap();
            claims
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_jwt_create,
    bench_jwt_verify,
    bench_bcrypt,
    bench_auth_flow_combined,
);
criterion_main!(benches);
