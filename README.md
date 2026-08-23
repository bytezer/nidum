# Nidum

> A highly opinionated, fast, and modular web framework for Rust, heavily inspired by NestJS.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

---

## 🎯 What is Nidum?

**Nidum** is a backend framework built on top of [Axum](https://github.com/tokio-rs/axum) and [Tokio](https://tokio.rs). It brings the enterprise-grade architecture, developer ergonomics, and structural predictability of TypeScript's NestJS (or Java's Spring Boot) to the Rust ecosystem.

### The Objective
Rust web frameworks (like Axum or Actix) are famously unopinionated—giving you complete freedom to structure your application. However, as teams and codebases scale, this freedom often leads to architectural spaghetti. 

**Nidum's goal is to lower the barrier of entry for enterprise teams migrating to Rust** by providing a familiar mental model (Controllers, Services, Modules, DI) while remaining strictly idiomatic. Through the use of powerful procedural macros and **Trait-Bound Dependency Injection**, Nidum guarantees your application's architectural integrity entirely at compile-time (Zero-Cost Abstractions).

---

## ✨ Features (What we have achieved)

Nidum has already successfully replicated the core pillars of an enterprise framework:

- 🛡️ **Compile-Time Safe Dependency Injection:** A completely static DI container. Dependencies are resolved using Trait Bounds (`Provides<T>`). If a service is missing, your app simply will not compile. Say goodbye to runtime DI panics.
- 📦 **Modular Architecture (`#[module]`):** Encapsulate related controllers and providers into strict boundaries to build scalable monoliths or microservices.
- 🚦 **Declarative Controllers (`#[controller]`):** Define HTTP endpoints cleanly using attribute macros (`#[get]`, `#[post]`, etc.) directly on your `impl` blocks.
- ✅ **Validation Pipes (`ValidJson<T>`):** Automatic validation of incoming HTTP payloads using the `validator` crate. Invalid DTOs automatically return structured `400 Bad Request` responses.
- 🔒 **Authorization Guards (`#[use_guards]`):** Apply declarative access-control layers to individual routes or entire controllers.
- 🕵️ **Interceptors (`#[use_interceptors]`):** A-O-P (Aspect-Oriented Programming) style request/response interception for global logging, caching, and response mapping.
- 🧯 **Global Exception Filters:** Centralized error handling to map unhandled Rust errors to custom JSON HTTP responses globally via `app.use_global_filter()`.
- 🗄️ **Database Agnostic DI:** Easily inject asynchronously initialized dependencies (like SeaORM or SQLx connection pools) natively into your modules via container pre-population (`.with_*()` methods).
- ⚙️ **Config Module:** A built-in `ConfigService` (`nidum-config`) to seamlessly load and parse strongly-typed environment variables from `.env` files.
- ⏰ **Task Scheduling (`#[cron]`):** Run background tasks inside services natively using Tokio, utilizing a syntax like `#[cron("0/5 * * * * * *")]` and centralized registry via `tasks = [...]`.
- 📡 **Local Event Emitter:** An in-memory pub/sub mechanism to decouple application logic, allowing services to emit strongly-typed events and react via `#[on_event]`.

---

## 🚀 Future Roadmap (What we aim to achieve)

- [ ] **Nidum CLI (`nidum-cli`):** A command-line tool similar to `nest-cli` to quickly scaffold projects, modules, controllers, and services without boilerplate.
- [ ] **OpenAPI / Swagger Auto-generation:** Analyze the `#[controller]` macros at compile-time to automatically generate a `swagger.json` file and host a Swagger UI route (`/api/docs`).
- [ ] **WebSocket Gateways:** A declarative abstraction over Axum's WebSockets to easily handle real-time events using `#[subscribe("event_name")]`.

---

## 💻 Quick Example

Here is a glimpse of how clean and predictable a Nidum application looks:

```rust
use nidum::prelude::*;
use std::sync::Arc;

// 1. Data Transfer Object with Validation Rules
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 3))]
    pub name: String,
    #[validate(email)]
    pub email: String,
}

// 2. Service Layer with Compile-Time DI
#[derive(Injectable)]
pub struct UserService {
    repo: Inject<UserRepository>,     // Automatically resolved at compile-time
    config: Inject<ConfigService>,    // Injected from nidum-config
}

impl UserService {
    pub fn create_user(&self, dto: CreateUserDto) {
        let app_name = self.config.get_or_default("APP_NAME", "Default");
        // Business logic...
    }
}

// 3. Controller Layer
#[derive(Injectable, Clone)]
pub struct UserController {
    service: Inject<UserService>,
}

#[controller("/users")]
impl UserController {
    // Automatically validates payload, returning 400 on failure
    #[post("/")]
    pub async fn create(&self, ValidJson(payload): ValidJson<CreateUserDto>) -> Result<StatusCode, HttpError> {
        self.service.create_user(payload);
        Ok(StatusCode::CREATED)
    }

    // Declaratively protected by an AuthGuard
    #[delete("/:id")]
    #[use_guards(AuthGuard)]
    pub async fn delete(&self, Path(id): Path<u64>) -> Result<StatusCode, HttpError> {
        Ok(StatusCode::NO_CONTENT)
    }
}

// 4. Feature Module
#[module(
    controllers = [UserController],
    providers = [UserService, UserRepository, AuthGuard, ConfigService],
)]
pub struct UserModule;

// 5. Bootstrap
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = NidumFactory::create()
        .register_module::<UserModule>(Arc::new(UserModuleContainer::new()));
        
    app.listen("127.0.0.1:3000").await?;
    Ok(())
}
```
