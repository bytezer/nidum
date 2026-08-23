# Nidum Architecture & Design Principles

Welcome to the internal documentation of Nidum. This document is meant for contributors and advanced users who want to understand *why* Nidum is built the way it is, and how we translate the ergonomics of dynamic languages (like TypeScript/NestJS) into the strict, memory-safe, compile-time world of Rust.

## 1. The Core Philosophy
Nidum is designed around three pillars:
1. **Familiar Mental Model:** Controllers, Providers, Modules, and Decorators (Macros).
2. **Compile-Time Safety:** If it compiles, the architecture is sound. No runtime missing dependency panics.
3. **Zero-Cost Abstractions:** The framework should not introduce dynamic overhead (like runtime reflection or massive HashMaps for DI).

## 2. Dependency Injection (Static vs Dynamic)
In frameworks like NestJS, Dependency Injection (DI) relies on **Runtime Reflection**. The framework scans your classes at startup, reads their metadata, and dynamically resolves dependencies from a central dictionary.

Rust **does not have runtime reflection**. Therefore, Nidum implements **Compile-Time Static DI** using Trait Bounds.

- When you write `providers = [UserService]` in a `#[module]`, Nidum's procedural macros generate a custom `Container` struct specifically for that module.
- It generates an implementation of `Provides<UserService> for Container`.
- When a controller requires `Inject<UserService>`, the compiler checks if the `Container` implements `Provides<UserService>`.
- **Result:** Missing dependencies are caught instantly by the Rust compiler.

## 3. Dealing with Asynchronous Initialization (Databases)
Static DI poses a challenge: How do we inject a Database Connection Pool that requires `.await` to connect, if our Container is generated statically by a macro?

**The Solution (`.with_*` Builder Pattern):**
Our `#[module]` macro automatically generates builder methods for every provider.
If you have `UsersDatabase`, the macro generates `.with_usersdatabase()`.
This allows you to initialize your async pool in `main()`, and push it into the container *before* the application starts, keeping the framework completely ORM-agnostic.

## 4. Background Tasks & Event Emitters
In NestJS, a decorator like `@Cron()` is magically discovered by the runtime. In Rust, an attribute macro on a method cannot automatically register itself globally because the macro only has access to the local AST, not the entire module tree.

**The Solution (`tasks = [...]` and `listeners = [...]`):**
To avoid polluting every struct with default trait implementations or requiring complex registry lookups, we use explicit registration arrays in the `#[module]` macro.

- `#[tasks]` on an `impl` block generates a `TaskRegistrar` trait implementation.
- You explicitly pass `tasks = [MyService]` in your module.
- The `#[module]` macro generates the exact code needed to pull the `SchedulerRegistry` from the DI container and register the tasks of `MyService`.

This requires writing the service name twice (`providers` and `tasks`), but in exchange, we avoid reflection completely, resulting in an application that consumes mere megabytes of RAM and boots in milliseconds.

## 5. Module Isolation (Bubbles)
In Nidum, DI is scoped per module. If two modules declare `SchedulerRegistry` in their providers, they will each get their own isolated instance. This enforces strict micro-architecture boundaries. Modules can be completely detached and moved to separate microservices without internal refactoring.
