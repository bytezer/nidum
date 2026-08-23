use axum::response::Response;

/// Trait to implement a Global Exception Filter.
pub trait ExceptionFilter: Send + Sync + 'static {
    fn catch(&self, response: Response) -> Response;
}
