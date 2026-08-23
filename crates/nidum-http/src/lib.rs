pub mod server;
pub mod extract;

// Re-export core structs
pub use server::HttpServer;

// Re-export common axum types and extractors to keep it unified
pub use axum::extract::{Path, Query, Json};
pub use axum::http::StatusCode;
pub use axum::response::{IntoResponse, Response};
pub use axum::routing;
pub use axum::Router;

// Re-export our custom extractors
pub use extract::ValidJson;
