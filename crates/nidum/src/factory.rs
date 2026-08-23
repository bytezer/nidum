use nidum_http::HttpServer;

pub struct NidumFactory;

impl NidumFactory {
    pub fn create() -> HttpServer {
        HttpServer::new()
    }
}
