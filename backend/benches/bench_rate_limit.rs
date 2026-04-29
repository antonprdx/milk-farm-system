use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_rate_limiter_in_memory(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("rate_limit_in_memory");

    group.bench_function("check_single_key", |b| {
        b.to_async(&rt).iter(|| {
            let limiter = milk_farm_backend::middleware::rate_limit::RateLimiter::new(
                1_000_000,
                60,
                None,
            );
            async move {
                limiter.check("192.168.1.1").await.unwrap();
            }
        });
    });

    group.bench_function("check_100_keys_concurrent", |b| {
        b.to_async(&rt).iter(|| {
            let limiter = Arc::new(
                milk_farm_backend::middleware::rate_limit::RateLimiter::new(
                    1_000_000,
                    60,
                    None,
                ),
            );
            async move {
                let mut handles = Vec::new();
                for i in 0..100 {
                    let limiter = limiter.clone();
                    handles.push(tokio::spawn(async move {
                        limiter.check(&format!("10.0.0.{}", i)).await
                    }));
                }
                for h in handles {
                    h.await.unwrap().unwrap();
                }
            }
        });
    });

    group.bench_function("check_1000_same_key", |b| {
        b.to_async(&rt).iter(|| {
            let limiter = milk_farm_backend::middleware::rate_limit::RateLimiter::new(
                1_000_000,
                60,
                None,
            );
            async move {
                for _ in 0..1000 {
                    limiter.check("192.168.1.1").await.unwrap();
                }
            }
        });
    });
    group.finish();
}

fn bench_rate_limiter_different_window_sizes(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("rate_limit_window_sizes");

    for (max, window) in [(10, 10), (100, 60), (1000, 3600)] {
        group.bench_function(format!("max_{max}_window_{window}s"), |b| {
            b.to_async(&rt).iter(|| {
                let limiter = milk_farm_backend::middleware::rate_limit::RateLimiter::new(
                    max, window, None,
                );
                async move {
                    limiter.check("192.168.1.1").await.unwrap();
                }
            });
        });
    }
    group.finish();
}

fn bench_rate_limiter_high_concurrency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("rate_limit_concurrency");
    group.sample_size(10);

    for concurrency in [10, 50, 100, 500] {
        group.bench_function(format!("{}_concurrent_ips", concurrency), |b| {
            b.to_async(&rt).iter(|| {
                let limiter = Arc::new(
                    milk_farm_backend::middleware::rate_limit::RateLimiter::new(
                        1_000_000,
                        60,
                        None,
                    ),
                );
                async move {
                    let mut handles = Vec::new();
                    for i in 0..concurrency {
                        let limiter = limiter.clone();
                        handles.push(tokio::spawn(async move {
                            limiter.check(&format!("10.0.{}.{i}", i / 256)).await
                        }));
                    }
                    for h in handles {
                        h.await.unwrap().unwrap();
                    }
                }
            });
        });
    }
    group.finish();
}

fn bench_extract_client_ip(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_client_ip");

    let headers_no_proxy = axum::http::HeaderMap::new();
    let mut headers_with_proxy = axum::http::HeaderMap::new();
    headers_with_proxy.insert(
        "X-Forwarded-For",
        "203.0.113.1, 70.41.3.18, 150.172.238.178"
            .parse()
            .unwrap(),
    );
    let mut headers_with_real_ip = axum::http::HeaderMap::new();
    headers_with_real_ip.insert("X-Real-IP", "203.0.113.50".parse().unwrap());

    group.bench_function("no_proxy_headers", |b| {
        b.iter(|| {
            milk_farm_backend::middleware::rate_limit::extract_client_ip(&headers_no_proxy, false)
        });
    });

    group.bench_function("with_x_forwarded_for", |b| {
        b.iter(|| {
            milk_farm_backend::middleware::rate_limit::extract_client_ip(&headers_with_proxy, true)
        });
    });

    group.bench_function("with_x_real_ip", |b| {
        b.iter(|| {
            milk_farm_backend::middleware::rate_limit::extract_client_ip(
                &headers_with_real_ip,
                true,
            )
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_rate_limiter_in_memory,
    bench_rate_limiter_different_window_sizes,
    bench_rate_limiter_high_concurrency,
    bench_extract_client_ip,
);
criterion_main!(benches);
