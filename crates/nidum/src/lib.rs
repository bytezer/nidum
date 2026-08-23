// Re-export core modules
pub mod factory;

#[doc(hidden)]
pub mod __private {
    pub use nidum_core;
    pub use nidum_http;
    pub use axum;
}

pub mod prelude {
    // 1. Core Traits and Types
    pub use nidum_core::{
        Controller, DiError, HttpError, Inject, Injectable, Module, CanActivate, Provides
    };

    // 2. HTTP Engine & Extractors
    pub use nidum_http::{
        IntoResponse, Json, Path, Query, Response, StatusCode, ValidJson
    };

    // 3. Proc-Macros
    pub use nidum_macros::{
        controller, delete, get, head, module, options, patch, post, put, use_guards, Injectable,
    };

    // 4. External Dependencies
    pub use serde::{Deserialize, Serialize};
    pub use validator::Validate;

    // 5. Factory
    pub use crate::factory::NidumFactory;
    
    // Axum types
    pub use axum::extract::Request as AxumRequest;
    pub use axum::async_trait;
}
