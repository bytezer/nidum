use axum::Router;
use std::sync::Arc;

/// A Module in the Trait-Bound DI system acts as a static container.
/// It provides a router configured with all its controllers and imported routers.
pub trait Module: Send + Sync + 'static {
    type Container;
    
    /// Returns the built router for this module
    fn build_router(container: Arc<Self::Container>) -> Router;
}
