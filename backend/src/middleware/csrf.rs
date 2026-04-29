use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, Response, StatusCode};
use axum::response::IntoResponse;
use std::pin::Pin;
use std::sync::Arc;
use tower::{Layer, Service};

use crate::config::Config;

pub struct CsrfGuard {
    origins: Vec<String>,
}

impl CsrfGuard {
    pub fn new(config: &Config) -> Self {
        Self {
            origins: config.cors_origins.clone(),
        }
    }

    fn validate(&self, headers: &HeaderMap) -> bool {
        let origin = headers
            .get("Origin")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if origin.is_empty() {
            return true;
        }

        self.origins
            .iter()
            .any(|allowed| origin == allowed || origin.starts_with(&format!("{}/", allowed)))
    }
}

#[derive(Clone)]
pub struct CsrfLayer {
    guard: Arc<CsrfGuard>,
}

impl CsrfLayer {
    pub fn new(config: &Config) -> Self {
        Self {
            guard: Arc::new(CsrfGuard::new(config)),
        }
    }
}

impl<S> Layer<S> for CsrfLayer {
    type Service = CsrfService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CsrfService {
            inner,
            guard: self.guard.clone(),
        }
    }
}

type BoxFuture<T, E> = Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>;

pub struct CsrfService<S> {
    inner: S,
    guard: Arc<CsrfGuard>,
}

impl<S> Clone for CsrfService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            guard: self.guard.clone(),
        }
    }
}

impl<S> Service<Request<Body>> for CsrfService<S>
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
        let method = req.method().clone();
        let guard = self.guard.clone();

        let is_state_changing = matches!(
            method,
            Method::POST | Method::PUT | Method::DELETE | Method::PATCH
        );

        if is_state_changing && !guard.validate(req.headers()) {
            return Box::pin(async { Ok(StatusCode::FORBIDDEN.into_response()) });
        }

        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);
        Box::pin(async move { inner.call(req).await })
    }
}
