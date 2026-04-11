use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Mutex;
use std::time::Instant;

use axum::body::Body;
use axum::http::{Method, Request, Response, StatusCode};
use axum::response::IntoResponse;
use tower::{Layer, Service};

use crate::errors::AppError;

pub struct RateLimitEntry {
    pub count: u32,
    pub window_start: Instant,
}

pub struct RateLimiter {
    max: u32,
    window_secs: u64,
    entries: Mutex<HashMap<String, RateLimitEntry>>,
}

impl RateLimiter {
    pub fn new(max: u32, window_secs: u64) -> Self {
        Self {
            max,
            window_secs,
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, key: &str) -> Result<(), AppError> {
        let mut map = self
            .entries
            .lock()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;
        let now = Instant::now();
        if let Some(entry) = map.get_mut(key) {
            if now.duration_since(entry.window_start).as_secs() > self.window_secs {
                entry.count = 1;
                entry.window_start = now;
            } else if entry.count >= self.max {
                return Err(AppError::RateLimited);
            } else {
                entry.count += 1;
            }
        } else {
            map.insert(
                key.to_string(),
                RateLimitEntry {
                    count: 1,
                    window_start: now,
                },
            );
        }
        map.retain(|_, v| now.duration_since(v.window_start).as_secs() <= self.window_secs * 2);
        Ok(())
    }
}

pub fn extract_client_ip(headers: &axum::http::HeaderMap, trust_proxy: bool) -> String {
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
    max_requests: u32,
    window_secs: u64,
    trust_proxy: bool,
}

impl RateLimitLayer {
    pub fn new(max_requests: u32, window_secs: u64, trust_proxy: bool) -> Self {
        Self {
            max_requests,
            window_secs,
            trust_proxy,
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            max_requests: self.max_requests,
            window_secs: self.window_secs,
            trust_proxy: self.trust_proxy,
            state: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

type BoxFuture<T, E> = Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>;

pub struct RateLimitService<S> {
    inner: S,
    max_requests: u32,
    window_secs: u64,
    trust_proxy: bool,
    state: std::sync::Arc<Mutex<HashMap<String, InternalEntry>>>,
}

struct InternalEntry {
    count: u32,
    window_start: Instant,
}

impl<S> Clone for RateLimitService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            max_requests: self.max_requests,
            window_secs: self.window_secs,
            trust_proxy: self.trust_proxy,
            state: self.state.clone(),
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
        let is_mutating = matches!(*req.method(), Method::POST | Method::PUT | Method::DELETE);

        if !is_mutating {
            let inner = self.inner.clone();
            let mut inner = std::mem::replace(&mut self.inner, inner);
            return Box::pin(async move { inner.call(req).await });
        }

        let key = extract_client_ip_from_request(&req, self.trust_proxy);

        let allowed = {
            let mut map = match self.state.lock() {
                Ok(m) => m,
                Err(_) => {
                    return Box::pin(async {
                        Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                    });
                }
            };
            let now = Instant::now();
            let max = self.max_requests;
            let window = self.window_secs;

            if let Some(entry) = map.get_mut(&key) {
                if now.duration_since(entry.window_start).as_secs() > window {
                    entry.count = 1;
                    entry.window_start = now;
                    true
                } else if entry.count >= max {
                    false
                } else {
                    entry.count += 1;
                    true
                }
            } else {
                map.insert(
                    key,
                    InternalEntry {
                        count: 1,
                        window_start: now,
                    },
                );
                map.retain(|_, v| now.duration_since(v.window_start).as_secs() <= window * 2);
                true
            }
        };

        if !allowed {
            return Box::pin(async { Ok(StatusCode::TOO_MANY_REQUESTS.into_response()) });
        }

        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);
        Box::pin(async move { inner.call(req).await })
    }
}

fn extract_client_ip_from_request(req: &Request<Body>, trust_proxy: bool) -> String {
    if trust_proxy
        && let Some(forwarded) = req.headers().get("X-Forwarded-For")
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
