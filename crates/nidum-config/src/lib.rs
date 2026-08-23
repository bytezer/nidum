use std::sync::Arc;
use dotenvy::dotenv;
use nidum_core::{Injectable, Provides, Module};

/// A service to access environment variables loaded from `.env` files or the system.
#[derive(Debug, Clone)]
pub struct ConfigService;

impl ConfigService {
    pub fn get<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        std::env::var(key).ok().and_then(|v| v.parse().ok())
    }

    pub fn get_or_default<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
        self.get(key).unwrap_or(default)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get::<String>(key)
    }
}

// Manually implement Injectable so we can load the .env file during creation
impl<C> Injectable<C> for ConfigService {
    fn create(_container: &C) -> Arc<Self> {
        let _ = dotenv(); // Load .env
        Arc::new(Self)
    }
}

pub struct ConfigModuleContainer {
    config_service: std::sync::OnceLock<Arc<ConfigService>>,
}

impl ConfigModuleContainer {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            config_service: std::sync::OnceLock::new(),
        })
    }
}

impl Provides<ConfigService> for ConfigModuleContainer {
    fn provide(&self) -> Arc<ConfigService> {
        self.config_service.get_or_init(|| ConfigService::create(self)).clone()
    }
}

pub struct ConfigModule;

impl Module for ConfigModule {
    type Container = ConfigModuleContainer;

    fn build_router(_container: Arc<Self::Container>) -> axum::Router {
        axum::Router::new()
    }
}
