use std::sync::Arc;
use axum::Router;

/// Trait implemented by all Nidum controllers.
/// Configures routes using the typed Container `C`.
pub trait Controller<C>: Send + Sync + 'static {
    fn base_path() -> &'static str;
    fn build_router(self: Arc<Self>, container: &C) -> Router;
}
