use std::pin::Pin;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Instant;

use axum::body::Body;
use axum::http::Request;
use prometheus::{Encoder, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder};
use tower::Service;

type BoxFuture<T, E> = Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>;

pub static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

pub static HTTP_REQUESTS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    IntCounterVec::new(
        Opts::new("http_requests_total", "Total number of HTTP requests"),
        &["method", "route", "status"],
    )
    .unwrap()
});

pub static HTTP_REQUEST_DURATION: LazyLock<HistogramVec> = LazyLock::new(|| {
    HistogramVec::new(
        prometheus::HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
        )
        .buckets(vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ]),
        &["method", "route"],
    )
    .unwrap()
});

pub fn init() {
    REGISTRY
        .register(Box::new(HTTP_REQUESTS_TOTAL.clone()))
        .unwrap();
    REGISTRY
        .register(Box::new(HTTP_REQUEST_DURATION.clone()))
        .unwrap();
}

pub fn gather() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap_or_default()
}

#[derive(Clone)]
pub struct MetricsLayer;

impl<S> tower::Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsService {
            inner,
            route_patterns: Arc::new(vec![
                "/api/v1/animals",
                "/api/v1/milk",
                "/api/v1/reproduction",
                "/api/v1/feed",
                "/api/v1/fitness",
                "/api/v1/grazing",
                "/api/v1/contacts",
                "/api/v1/bulk-tank",
                "/api/v1/reports",
                "/api/v1/settings",
                "/api/v1/analytics",
                "/api/v1/locations",
                "/api/v1/auth",
                "/api/v1/health",
                "/api/v1/healthz",
                "/api/v1/readyz",
                "/api/v1/stats",
                "/api/v1/docs",
            ]),
        }
    }
}

pub struct MetricsService<S> {
    inner: S,
    route_patterns: Arc<Vec<&'static str>>,
}

impl<S> Clone for MetricsService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            route_patterns: self.route_patterns.clone(),
        }
    }
}

fn normalize_route(path: &str, patterns: &[&'static str]) -> String {
    for pattern in patterns {
        if path.starts_with(pattern) {
            return pattern.to_string();
        }
    }
    path.to_string()
}

impl<S> Service<Request<Body>> for MetricsService<S>
where
    S: Service<Request<Body>, Response = axum::http::Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = axum::http::Response<Body>;
    type Error = S::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let path = req.uri().path().to_string();
        let method = req.method().clone();
        let route = normalize_route(&path, &self.route_patterns);

        let start = Instant::now();
        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);

        Box::pin(async move {
            let result = inner.call(req).await;
            let elapsed = start.elapsed().as_secs_f64();

            let status = match &result {
                Ok(resp) => resp.status().as_u16().to_string(),
                Err(_) => "500".to_string(),
            };

            HTTP_REQUESTS_TOTAL
                .with_label_values(&[method.as_str(), &route, &status])
                .inc();

            HTTP_REQUEST_DURATION
                .with_label_values(&[method.as_str(), &route])
                .observe(elapsed);

            result
        })
    }
}
