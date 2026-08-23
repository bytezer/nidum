use nidum::prelude::*;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, Level};

#[derive(Injectable)]
pub struct LoggingInterceptor;

#[async_trait]
impl Interceptor for LoggingInterceptor {
    async fn intercept(&self, req: AxumRequest, next: Next) -> Response {
        let method = req.method().clone();
        let uri = req.uri().clone();
        
        let start = Instant::now();
        info!("--> {} {}", method, uri);

        let response = next.run(req).await;

        let duration = start.elapsed();
        let status = response.status();
        info!("<-- {} {} {} {:?}", method, uri, status, duration);

        response
    }
}

pub struct HttpExceptionFilter;

impl ExceptionFilter for HttpExceptionFilter {
    fn catch(&self, response: Response) -> Response {
        let status = response.status();
        
        let body = serde_json::json!({
            "statusCode": status.as_u16(),
            "message": "An error occurred, caught by global filter",
            "timestamp": "2026-08-23"
        });

        Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(nidum::__private::axum::body::Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap()
    }
}

#[derive(Injectable, Clone)]
pub struct AppController;

#[controller("/")]
#[use_interceptors(LoggingInterceptor)]
impl AppController {
    #[get("/hello")]
    pub async fn hello(&self) -> &'static str {
        "Hello with Interceptors!"
    }

    #[get("/error")]
    pub async fn error(&self) -> Result<&'static str, StatusCode> {
        Err(StatusCode::BAD_REQUEST)
    }
}

#[module(
    controllers = [AppController],
    providers = [LoggingInterceptor],
)]
pub struct AppModule;

#[nidum::zero]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let app = NidumFactory::create()
        .register_module::<AppModule>(Arc::new(AppModuleContainer::new()))
        .use_global_filter(HttpExceptionFilter);

    info!("Starting server on 127.0.0.1:3000");
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
