use nidum::prelude::*;
use std::sync::Arc;
use nidum::__private::tokio::time::{sleep, Duration};

// 1. Define typed events
#[derive(Clone, Debug)]
pub struct UserCreatedEvent {
    pub user_id: u64,
    pub email: String,
}

// 2. A service that emits events
#[derive(Injectable)]
pub struct UserService {
    events: Inject<EventEmitter>,
}

impl UserService {
    pub fn create_user(&self, email: &str) {
        println!("User created with email: {}", email);
        
        // Fire and forget
        self.events.emit(UserCreatedEvent {
            user_id: 42,
            email: email.to_string(),
        });
    }
}

// 3. A service that listens to events
#[derive(Injectable)]
pub struct EmailService;

#[events]
impl EmailService {
    #[on_event]
    pub async fn handle_user_created(&self, event: UserCreatedEvent) {
        println!("Received UserCreatedEvent: sending welcome email to {}", event.email);
        sleep(Duration::from_millis(100)).await;
        println!("Email sent to {}!", event.email);
    }
}

// 4. Controller to trigger the creation
#[derive(Injectable, Clone)]
pub struct AppController {
    user_service: Inject<UserService>,
}

#[controller("/")]
impl AppController {
    #[post("/create_user")]
    pub async fn create(&self) -> &'static str {
        self.user_service.create_user("test@nidum.rs");
        "User creation initiated"
    }
}

// 5. Module: note the `listeners` array
#[module(
    controllers = [AppController],
    providers = [EventEmitter, UserService, EmailService],
    listeners = [EmailService]
)]
pub struct AppModule;

#[zero]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = NidumFactory::create()
        .register_module::<AppModule>(Arc::new(AppModuleContainer::new()));

    println!("Starting server on 127.0.0.1:3000");
    println!("To test: curl -X POST http://127.0.0.1:3000/create_user");
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
