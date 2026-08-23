// Re-export core modules
pub mod factory;

#[doc(hidden)]
pub mod __private {
    pub use nidum_core;
    pub use nidum_http;
    pub use axum;
    pub use tokio;
    pub use nidum_core::chrono;
    pub use nidum_core::cron;
}

pub mod prelude {
    // Core Traits and Types
    pub use nidum_core::{
        Controller, DiError, HttpError, Inject, Injectable, Module, CanActivate, Interceptor, Provides, ExceptionFilter, EventEmitter, EventListener, SchedulerRegistry, TaskRegistrar
    };

    // HTTP Engine & Extractors
    pub use nidum_http::{
        IntoResponse, Json, Path, Query, Response, StatusCode, ValidJson
    };

    // Proc-Macros
    pub use nidum_macros::{
        controller, delete, get, head, module, options, patch, post, put, use_guards, use_interceptors, events, tasks, cron, Injectable,
    };

    // Built-in Modules
    pub use nidum_config::{ConfigModule, ConfigService};

    // External Dependencies
    pub use serde::{Deserialize, Serialize};
    pub use validator::Validate;

    // Factory
    pub use crate::factory::NidumFactory;
    
    // Axum types
    pub use axum::extract::Request as AxumRequest;
    pub use axum::middleware::Next;
    pub use axum::async_trait;
}
