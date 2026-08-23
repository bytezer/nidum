use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use nidum_core::Module;
use tower_http::trace::TraceLayer;
use tracing::info;

/// HTTP Server runner for Nidum applications.
pub struct HttpServer {
    router: Router,
    global_prefix: Option<String>,
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            global_prefix: None,
        }
    }

    pub fn set_global_prefix(mut self, prefix: impl Into<String>) -> Self {
        let p = prefix.into();
        let trimmed = p.trim_matches('/');
        self.global_prefix = if trimmed.is_empty() {
            None
        } else {
            Some(format!("/{}", trimmed))
        };
        self
    }

    pub fn register_module<M: Module>(mut self, container: Arc<M::Container>) -> Self {
        let module_router = M::build_router(container);
        self.router = self.router.merge(module_router);
        self
    }

    pub fn build_router(self) -> Router {
        let final_router = if let Some(prefix) = self.global_prefix {
            Router::new().nest(&prefix, self.router)
        } else {
            self.router
        };

        final_router.layer(TraceLayer::new_for_http())
    }

    pub async fn listen(self, addr: impl Into<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr_str = addr.into();
        let socket_addr: SocketAddr = addr_str.parse()?;
        let app = self.build_router();

        info!("🚀 Nidum application started and listening on http://{}", socket_addr);

        let listener = tokio::net::TcpListener::bind(socket_addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}
