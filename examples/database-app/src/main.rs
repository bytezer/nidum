use nidum::prelude::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct DatabaseConnection {
    pub url: String,
}

impl DatabaseConnection {
    pub async fn connect(url: &str) -> Self {
        println!("Connecting to database at {}...", url);
        sleep(Duration::from_millis(500)).await;
        println!("Connected!");
        Self { url: url.to_string() }
    }
}

#[derive(Clone)]
pub struct UsersDatabase(pub DatabaseConnection);

impl<C> Injectable<C> for UsersDatabase {
    fn create(_: &C) -> Arc<Self> {
        panic!("UsersDatabase must be pre-injected using with_usersdatabase()")
    }
}

#[derive(Clone)]
pub struct AnalyticsDatabase(pub DatabaseConnection);

impl<C> Injectable<C> for AnalyticsDatabase {
    fn create(_: &C) -> Arc<Self> {
        panic!("AnalyticsDatabase must be pre-injected using with_analyticsdatabase()")
    }
}

#[derive(Injectable)]
pub struct UserRepository {
    db: Inject<UsersDatabase>,
}

impl UserRepository {
    pub fn get_users(&self) -> Vec<&'static str> {
        println!("Querying users from: {}", self.db.0.0.url);
        vec!["Alice", "Bob", "Charlie"]
    }
}

#[derive(Injectable)]
pub struct AnalyticsRepository {
    db: Inject<AnalyticsDatabase>,
}

impl AnalyticsRepository {
    pub fn get_stats(&self) -> &'static str {
        println!("Querying stats from: {}", self.db.0.0.url);
        "1000 visitors today"
    }
}

#[derive(Injectable, Clone)]
pub struct AppController {
    user_repo: Inject<UserRepository>,
    analytics_repo: Inject<AnalyticsRepository>,
}

#[controller("/")]
impl AppController {
    #[get("/users")]
    pub async fn get_users(&self) -> Json<Vec<&'static str>> {
        Json(self.user_repo.get_users())
    }

    #[get("/stats")]
    pub async fn get_stats(&self) -> &'static str {
        self.analytics_repo.get_stats()
    }
}

#[module(
    controllers = [AppController],
    providers = [UserRepository, AnalyticsRepository, UsersDatabase, AnalyticsDatabase],
)]
pub struct AppModule;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let users_conn = DatabaseConnection::connect("postgres://localhost/users").await;
    let analytics_conn = DatabaseConnection::connect("postgres://localhost/analytics").await;

    let container = Arc::new(
        AppModuleContainer::new()
            .with_usersdatabase(Arc::new(UsersDatabase(users_conn)))
            .with_analyticsdatabase(Arc::new(AnalyticsDatabase(analytics_conn)))
    );

    let app = NidumFactory::create().register_module::<AppModule>(container);

    println!("Starting server on 127.0.0.1:3000");
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
