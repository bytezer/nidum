use axum::{extract::Request, middleware::Next, response::Response};
use std::sync::Arc;
use async_trait::async_trait;

/// Trait to implement an Interceptor.
#[async_trait]
pub trait Interceptor: Send + Sync + 'static {
    async fn intercept(&self, req: Request, next: Next) -> Response;
}

/// Axum middleware wrapper that extracts an `Interceptor` from the state (container)
/// and calls it.
pub async fn interceptor_middleware<I: Interceptor>(
    state: axum::extract::State<Arc<I>>,
    req: Request,
    next: Next,
) -> Response {
    state.0.intercept(req, next).await
}
