# Nidum

> A fast, intuitive, and modular web framework for Rust inspired by NestJS.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

Nidum brings the ergonomics, modular architecture, and developer experience of NestJS to the safety and blazing performance of Rust and Tokio.

---

## 🌟 Key Highlights

- **Modular Architecture (`#[module]`):** Group related controllers, providers, and imports with automatic deduplication.
- **Dynamic Modules (`DynamicModule`):** Configurable modules in runtime.
- **Declarative Controllers (`#[controller]`):** Define REST endpoints using attribute macros (`#[get]`, `#[post]`, etc.).
- **Dependency Injection (DI):** Automatic resolution of service dependency trees using `#[derive(Injectable)]` and `Inject<T>`.
- **Validation Pipes (`ValidJson<T>`):** Automatic validation of incoming DTOs using the `validator` crate.
- **Guards (`#[use_guards(Guard)]`):** Declarative authorization and request filtering layers.
- **Structured Error Handling:** Standard `HttpError` enum with automatic JSON responses.

---

## 🚀 Quick Example

```rust
use nidum::prelude::*;

// 1. Data Transfer Object with Validation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 3))]
    pub name: String,
    #[validate(email)]
    pub email: String,
}

// 2. Auth Guard
#[derive(Injectable)]
pub struct AuthGuard;

#[async_trait]
impl CanActivate for AuthGuard {
    async fn can_activate(&self, req: &mut AxumRequest) -> Result<(), HttpError> {
        if req.headers().contains_key("Authorization") {
            Ok(())
        } else {
            Err(HttpError::Unauthorized("Missing token".to_string()))
        }
    }
}

// 3. Controller
#[derive(Injectable, Clone)]
pub struct UserController {
    service: Inject<UserService>, // Auto-injected!
}

#[controller("/users")]
impl UserController {
    // ValidJson automatically validates the payload and returns 400 Bad Request if invalid.
    #[post("/")]
    pub async fn create(&self, ValidJson(payload): ValidJson<CreateUserDto>) -> Result<StatusCode, HttpError> {
        self.service.create_user(payload).await;
        Ok(StatusCode::CREATED)
    }

    // Protected by AuthGuard
    #[delete("/:id")]
    #[use_guards(AuthGuard)]
    pub async fn delete(&self, Path(id): Path<u64>) -> Result<StatusCode, HttpError> {
        self.service.delete_user(id).await;
        Ok(StatusCode::NO_CONTENT)
    }
}

// 4. Feature Module
#[module(
    controllers = [UserController],
    providers = [UserService, UserRepository, AuthGuard],
)]
pub struct UserModule;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = NidumFactory::create().register_module::<UserModule>()?;
    app.listen("127.0.0.1:3000").await?;
    Ok(())
}
```
