use std::sync::Arc;

/// A smart pointer wrapper around `Arc<T>` used for Dependency Injection.
#[derive(Debug)]
pub struct Inject<T>(pub Arc<T>);

impl<T> Inject<T> {
    pub fn new(instance: Arc<T>) -> Self {
        Self(instance)
    }
}

impl<T> Clone for Inject<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> std::ops::Deref for Inject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<Arc<T>> for Inject<T> {
    fn from(arc: Arc<T>) -> Self {
        Self(arc)
    }
}
