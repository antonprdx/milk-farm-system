use std::pin::Pin;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, Request, Response, StatusCode};
use axum::response::IntoResponse;
use tower::{Layer, Service};

use crate::errors::AppError;

pub struct RateLimiter {
    max: u32,
    window_secs: u64,
    state: std::sync::Mutex<RateLimitState>,
}

struct RateLimitState {
    entries: std::collections::HashMap<String, RateLimitEntry>,
}

pub struct RateLimitEntry {
    pub count: u32,
    pub window_start: std::time::Instant,
}

impl RateLimiter {
    pub fn new(max: u32, window_secs: u64) -> Self {
        Self {
            max,
            window_secs,
            state: std::sync::Mutex::new(RateLimitState {
                entries: std::collections::HashMap::new(),
            }),
        }
    }

    pub fn check(&self, key: &str) -> Result<(), AppError> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;
        let now = std::time::Instant::now();
        if let Some(entry) = state.entries.get_mut(key) {
            if now.duration_since(entry.window_start).as_secs() > self.window_secs {
                entry.count = 1;
                entry.window_start = now;
            } else if entry.count >= self.max {
                return Err(AppError::RateLimited);
            } else {
                entry.count += 1;
            }
        } else {
            state.entries.insert(
                key.to_string(),
                RateLimitEntry {
                    count: 1,
                    window_start: now,
                },
            );
        }
        state
            .entries
            .retain(|_, v| now.duration_since(v.window_start).as_secs() <= self.window_secs * 2);
        Ok(())
    }
}

pub fn extract_client_ip(headers: &HeaderMap, trust_proxy: bool) -> String {
    if trust_proxy
        && let Some(forwarded) = headers.get("X-Forwarded-For")
        && let Ok(val) = forwarded.to_str()
        && let Some(ip) = val.split(',').next()
    {
        let ip = ip.trim().to_string();
        if !ip.is_empty() {
            return ip;
        }
    }
    "unknown".to_string()
}

#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Arc<RateLimiter>,
    trust_proxy: bool,
}

impl RateLimitLayer {
    pub fn new(max_requests: u32, window_secs: u64, trust_proxy: bool) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new(max_requests, window_secs)),
            trust_proxy,
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
            trust_proxy: self.trust_proxy,
        }
    }
}

type BoxFuture<T, E> = Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>;

pub struct RateLimitService<S> {
    inner: S,
    limiter: Arc<RateLimiter>,
    trust_proxy: bool,
}

impl<S> Clone for RateLimitService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            limiter: self.limiter.clone(),
            trust_proxy: self.trust_proxy,
        }
    }
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let key = extract_client_ip(req.headers(), self.trust_proxy);

        if self.limiter.check(&key).is_err() {
            return Box::pin(async { Ok(StatusCode::TOO_MANY_REQUESTS.into_response()) });
        }

        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);
        Box::pin(async move { inner.call(req).await })
    }
}
