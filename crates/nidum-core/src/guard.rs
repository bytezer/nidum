use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;
use axum::async_trait;

use crate::error::HttpError;

/// Trait implemented by authorization Guards.
#[async_trait]
pub trait CanActivate: Send + Sync + 'static {
    /// Determines whether a request should be processed.
    /// Returns `Ok(())` to allow the request to proceed, or `Err(HttpError)` to block it.
    async fn can_activate(&self, req: &mut Request) -> Result<(), HttpError>;
}

/// Helper middleware function that adapts any `CanActivate` into an Axum middleware layer.
pub async fn guard_middleware<G: CanActivate>(
    axum::extract::State(guard): axum::extract::State<Arc<G>>,
    mut req: Request,
    next: Next,
) -> Result<Response, HttpError> {
    guard.can_activate(&mut req).await?;
    Ok(next.run(req).await)
}
