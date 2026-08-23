use std::sync::Arc;

pub mod inject;

pub use inject::Inject;

/// Trait representing a container that provides a specific type `T`.
/// This is the core of Nidum's Compile-Time Trait-Bound DI.
pub trait Provides<T> {
    fn provide(&self) -> Arc<T>;
}

/// Trait for structs that can be instantiated by a typed container `C`.
pub trait Injectable<C>: Sized + Send + Sync + 'static {
    fn create(container: &C) -> Arc<Self>;
}

/// Error type for dynamic resolutions (kept for compatibility with dynamic modules if needed)
#[derive(Debug, thiserror::Error)]
pub enum DiError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
}
