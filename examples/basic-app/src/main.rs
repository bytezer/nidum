use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use nidum::prelude::*;
use nidum::__private::tokio::sync::RwLock;

// Data Transfer Objects (DTOs)

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 3, message = "Name must be at least 3 characters long"))]
    pub name: String,
    
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub search: Option<String>,
}


// Auth Feature (Guard)

#[derive(Injectable)]
pub struct AuthGuard;

#[async_trait]
impl CanActivate for AuthGuard {
    async fn can_activate(&self, req: &mut AxumRequest) -> Result<(), HttpError> {
        if let Some(auth_header) = req.headers().get("Authorization") {
            if auth_header == "Bearer super-secreto" {
                return Ok(());
            }
        }
        Err(HttpError::Unauthorized("Invalid or missing token".to_string()))
    }
}

// User Feature (Repository, Service, Controller, Module)

#[derive(Injectable, Default)]
pub struct UserRepository {
    #[inject(default)]
    users: Arc<RwLock<Vec<UserDto>>>,
    #[inject(default)]
    next_id: Arc<AtomicU64>,
}

impl UserRepository {
    pub async fn find_all(&self) -> Vec<UserDto> {
        self.users.read().await.clone()
    }

    pub async fn find_by_id(&self, id: u64) -> Option<UserDto> {
        let users = self.users.read().await;
        users.iter().find(|u| u.id == id).cloned()
    }

    pub async fn create(&self, name: String, email: String) -> UserDto {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let new_user = UserDto { id, name, email };
        let mut users = self.users.write().await;
        users.push(new_user.clone());
        new_user
    }

    pub async fn delete(&self, id: u64) -> bool {
        let mut users = self.users.write().await;
        let initial_len = users.len();
        users.retain(|u| u.id != id);
        users.len() < initial_len
    }
}

#[derive(Injectable)]
pub struct UserService {
    repo: Inject<UserRepository>,
    config: Inject<ConfigService>,
}

impl UserService {
    pub async fn list_users(&self, search: Option<String>) -> Vec<UserDto> {
        let app_name = self.config.get_or_default("APP_NAME", "DefaultApp".to_string());
        println!("Listing users for app: {}", app_name);

        let users = self.repo.find_all().await;
        if let Some(term) = search {
            users
                .into_iter()
                .filter(|u| u.name.to_lowercase().contains(&term.to_lowercase()))
                .collect()
        } else {
            users
        }
    }

    pub async fn get_user(&self, id: u64) -> Result<UserDto, HttpError> {
        self.repo
            .find_by_id(id)
            .await
            .ok_or_else(|| HttpError::NotFound(format!("User with id {} not found", id)))
    }

    pub async fn create_user(&self, dto: CreateUserDto) -> Result<UserDto, HttpError> {
        Ok(self.repo.create(dto.name, dto.email).await)
    }

    pub async fn delete_user(&self, id: u64) -> Result<(), HttpError> {
        let deleted = self.repo.delete(id).await;
        if deleted {
            Ok(())
        } else {
            Err(HttpError::NotFound(format!("User with id {} not found", id)))
        }
    }
}

#[derive(Injectable, Clone)]
pub struct UserController {
    service: Inject<UserService>,
}

#[controller("/users")]
impl UserController {
    #[get("/")]
    pub async fn list(&self, Query(query): Query<UserQuery>) -> Json<Vec<UserDto>> {
        Json(self.service.list_users(query.search).await)
    }

    #[get("/:id")]
    pub async fn get_by_id(&self, Path(id): Path<u64>) -> Result<Json<UserDto>, HttpError> {
        let user = self.service.get_user(id).await?;
        Ok(Json(user))
    }

    #[post("/")]
    pub async fn create(&self, ValidJson(payload): ValidJson<CreateUserDto>) -> Result<(StatusCode, Json<UserDto>), HttpError> {
        let user = self.service.create_user(payload).await?;
        Ok((StatusCode::CREATED, Json(user)))
    }

    #[delete("/:id")]
    #[use_guards(AuthGuard)]
    pub async fn delete(&self, Path(id): Path<u64>) -> Result<StatusCode, HttpError> {
        self.service.delete_user(id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}

// UserModule now requires ConfigService in its container. Since we have a flat module pattern currently,
// we just add ConfigService to the providers of AppModule, and since UserModule is part of the app,
// wait, we currently map providers locally. 
// Yes, our `#[module]` creates a Container for EXACTLY what is in `providers`.
// So if `UserService` needs `ConfigService`, it MUST be in `providers` of `UserModule` OR we flat-list it in `AppModule`.
#[module(
    controllers = [UserController],
    providers = [UserService, UserRepository, AuthGuard, ConfigService],
)]
pub struct UserModule;


// Health Feature (Controller & Module)

#[derive(Injectable, Clone, Default)]
pub struct HealthController;

#[controller("/health")]
impl HealthController {
    #[get("/")]
    pub async fn check(&self) -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "status": "healthy",
            "framework": "Nidum",
            "version": "0.1.0"
        }))
    }
}

#[module(
    controllers = [HealthController],
)]
pub struct HealthModule;

// Root Application Module (AppModule)

#[module(
    imports = [HealthModule, UserModule],
)]
pub struct AppModule;

// Bootstrap
#[zero]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug,basic_app=debug".into()),
        )
        .init();

    // Bootstrapping the entire application with its root AppModule
    let app = NidumFactory::create()
        .set_global_prefix("/api/v1")
        .register_module::<AppModule>(Arc::new(AppModuleContainer::new()));

    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
