use std::pin::Pin;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, Request, Response, StatusCode};
use axum::response::IntoResponse;
use tower::{Layer, Service};

use crate::errors::AppError;

struct RateLimitEntry {
    count: u32,
    window_start: std::time::Instant,
}

pub struct RateLimiter {
    max: u32,
    window_secs: u64,
    redis: Option<redis::aio::ConnectionManager>,
    fallback: tokio::sync::Mutex<std::collections::HashMap<String, RateLimitEntry>>,
}

impl RateLimiter {
    pub fn new(max: u32, window_secs: u64, redis: Option<redis::aio::ConnectionManager>) -> Self {
        Self {
            max,
            window_secs,
            redis,
            fallback: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub async fn check(&self, key: &str) -> Result<(), AppError> {
        if let Some(conn) = &self.redis {
            return self.check_redis(conn, key).await;
        }
        self.check_fallback(key).await
    }

    async fn check_redis(
        &self,
        conn: &redis::aio::ConnectionManager,
        key: &str,
    ) -> Result<(), AppError> {
        let redis_key = format!("rl:{key}");
        let mut conn = conn.clone();
        let count: i64 = redis::cmd("INCR")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                tracing::warn!("Redis rate limit error: {e}");
                AppError::Internal(anyhow::anyhow!(e.to_string()))
            })?;

        if count == 1 {
            let _: () = redis::cmd("EXPIRE")
                .arg(&redis_key)
                .arg(self.window_secs)
                .query_async(&mut conn)
                .await
                .unwrap_or(());
        }

        if count > self.max as i64 {
            crate::middleware::metrics::RATE_LIMIT_REJECTIONS.with_label_values(&[key]).inc();
            return Err(AppError::RateLimited);
        }
        Ok(())
    }

    async fn check_fallback(&self, key: &str) -> Result<(), AppError> {
        let mut state = self.fallback.lock().await;
        let now = std::time::Instant::now();
        if let Some(entry) = state.get_mut(key) {
            if now.duration_since(entry.window_start).as_secs() > self.window_secs {
                entry.count = 1;
                entry.window_start = now;
            } else if entry.count >= self.max {
                crate::middleware::metrics::RATE_LIMIT_REJECTIONS.with_label_values(&[key]).inc();
                return Err(AppError::RateLimited);
            } else {
                entry.count += 1;
            }
        } else {
            state.insert(
                key.to_string(),
                RateLimitEntry {
                    count: 1,
                    window_start: now,
                },
            );
        }
        state.retain(|_, v| now.duration_since(v.window_start).as_secs() <= self.window_secs * 2);
        Ok(())
    }
}

pub fn extract_client_ip(headers: &HeaderMap, trust_proxy: bool) -> String {
    if trust_proxy {
        if let Some(forwarded) = headers.get("X-Forwarded-For")
            && let Ok(val) = forwarded.to_str()
            && let Some(ip) = val.split(',').next()
        {
            let ip = ip.trim().to_string();
            if !ip.is_empty() {
                return ip;
            }
        }
        if let Some(real_ip) = headers.get("X-Real-IP")
            && let Ok(val) = real_ip.to_str()
        {
            let ip = val.trim().to_string();
            if !ip.is_empty() {
                return ip;
            }
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
    pub fn new(
        max_requests: u32,
        window_secs: u64,
        trust_proxy: bool,
        redis: Option<redis::aio::ConnectionManager>,
    ) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new(max_requests, window_secs, redis)),
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
        let limiter = self.limiter.clone();

        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);
        Box::pin(async move {
            if limiter.check(&key).await.is_err() {
                return Ok(StatusCode::TOO_MANY_REQUESTS.into_response());
            }
            inner.call(req).await
        })
    }
}
