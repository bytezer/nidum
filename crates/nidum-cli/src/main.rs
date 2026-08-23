use clap::{Parser, Subcommand};
use std::process::Command;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nidum", version = "0.1.0", about = "Nidum CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Nidum project
    New {
        /// The name of the project
        name: String,
    },
    /// Generate a Nidum element (module, controller, service)
    #[command(alias = "g")]
    Generate {
        #[command(subcommand)]
        component: GenerateComponent,
    },
}

#[derive(Subcommand)]
enum GenerateComponent {
    /// Generate a module
    #[command(alias = "mo")]
    Module {
        /// The name of the module (e.g. users)
        name: String,
    },
    /// Generate a controller
    #[command(alias = "co")]
    Controller {
        /// The name of the controller (e.g. users)
        name: String,
    },
    /// Generate a service
    #[command(alias = "s")]
    Service {
        /// The name of the service (e.g. users)
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name } => {
            handle_new(name);
        }
        Commands::Generate { component } => {
            match component {
                GenerateComponent::Module { name } => handle_generate_module(name),
                GenerateComponent::Controller { name } => handle_generate_controller(name),
                GenerateComponent::Service { name } => handle_generate_service(name),
            }
        }
    }
}

fn handle_new(name: &str) {
    println!("Creating new Nidum project: {}", name);
    
    let status = Command::new("cargo")
        .arg("new")
        .arg(name)
        .status()
        .expect("Failed to execute cargo new");

    if !status.success() {
        eprintln!("Failed to create cargo project.");
        return;
    }

    let cargo_toml_path = PathBuf::from(name).join("Cargo.toml");
    let mut cargo_toml = fs::read_to_string(&cargo_toml_path).expect("Failed to read Cargo.toml");
    
    // Removed duplicate dependencies block
    cargo_toml.push_str("nidum = { version = \"*\" }\n");
    
    
    fs::write(&cargo_toml_path, cargo_toml).expect("Failed to write Cargo.toml");

    let main_rs_path = PathBuf::from(name).join("src").join("main.rs");
    let boilerplate = r#"use nidum::prelude::*;
use std::sync::Arc;

#[derive(Injectable, Clone)]
pub struct AppController;

#[controller("/")]
impl AppController {
    #[get("/")]
    pub async fn hello(&self) -> &'static str {
        "Hello from Nidum!"
    }
}

#[module(
    controllers = [AppController],
    providers = []
)]
pub struct AppModule;

#[nidum::zero]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = NidumFactory::create()
        .register_module::<AppModule>(Arc::new(AppModuleContainer::new()));

    println!("Starting server on http://127.0.0.1:3000");
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
"#;
    fs::write(&main_rs_path, boilerplate).expect("Failed to write main.rs");

    println!("Project '{}' successfully created.", name);
    println!("To get started:");
    println!("  cd {}", name);
    println!("  cargo run");
}

fn handle_generate_module(name: &str) {
    let folder_path = PathBuf::from("src").join(name);
    fs::create_dir_all(&folder_path).expect("Failed to create directory");

    let mod_rs_path = folder_path.join("mod.rs");
    let struct_name = format!("{}Module", uppercase_first(name));
    
    let content = format!(r#"use nidum::prelude::*;

#[module(
    controllers = [],
    providers = []
)]
pub struct {};
"#, struct_name);

    fs::write(&mod_rs_path, content).expect("Failed to write mod.rs file");
    println!("Created {}", mod_rs_path.display());

    // Auto-wire in main.rs
    let main_rs_path = PathBuf::from("src").join("main.rs");
    if main_rs_path.exists() {
        let current_content = fs::read_to_string(&main_rs_path).unwrap_or_default();
        let mod_statement = format!("pub mod {};\n", name);
        if !current_content.contains(&mod_statement) {
            let new_content = format!("{}{}", mod_statement, current_content);
            fs::write(&main_rs_path, new_content).expect("Failed to update main.rs");
            println!("Updated src/main.rs (Registered module '{}')", name);
        }
    }
}

fn handle_generate_controller(name: &str) {
    let folder_path = PathBuf::from("src").join(name);
    fs::create_dir_all(&folder_path).expect("Failed to create directory");

    let file_name = format!("{}_controller.rs", name);
    let file_path = folder_path.join(&file_name);

    let struct_name = format!("{}Controller", uppercase_first(name));
    
    let content = format!(r#"use nidum::prelude::*;

#[derive(Injectable, Clone)]
pub struct {struct_name} {{
}}

#[controller("/{name}")]
impl {struct_name} {{
    #[get("/")]
    pub async fn index(&self) -> &'static str {{
        "{struct_name} works!"
    }}
}}
"#);

    fs::write(&file_path, content).expect("Failed to write controller file");
    println!("Created {}", file_path.display());

    // Auto-wire in mod.rs
    auto_wire_in_mod_rs(name, &format!("{}_controller", name));
}

fn handle_generate_service(name: &str) {
    let folder_path = PathBuf::from("src").join(name);
    fs::create_dir_all(&folder_path).expect("Failed to create directory");

    let file_name = format!("{}_service.rs", name);
    let file_path = folder_path.join(&file_name);

    let struct_name = format!("{}Service", uppercase_first(name));
    
    let content = format!(r#"use nidum::prelude::*;

#[derive(Injectable)]
pub struct {struct_name} {{
}}

impl {struct_name} {{
    pub fn do_something(&self) {{
        // Implementation
    }}
}}
"#);

    fs::write(&file_path, content).expect("Failed to write service file");
    println!("Created {}", file_path.display());

    // Auto-wire in mod.rs
    auto_wire_in_mod_rs(name, &format!("{}_service", name));
}

fn auto_wire_in_mod_rs(module_name: &str, component_name: &str) {
    let mod_rs_path = PathBuf::from("src").join(module_name).join("mod.rs");
    if !mod_rs_path.exists() {
        // If there's no mod.rs, we create a very basic one
        let content = format!("pub mod {};\n", component_name);
        fs::write(&mod_rs_path, content).expect("Failed to create mod.rs");
        println!("Created {}", mod_rs_path.display());
    } else {
        let current_content = fs::read_to_string(&mod_rs_path).unwrap_or_default();
        let mod_statement = format!("pub mod {};\n", component_name);
        if !current_content.contains(&mod_statement) {
            let new_content = format!("{}{}", mod_statement, current_content);
            fs::write(&mod_rs_path, new_content).expect("Failed to update mod.rs");
            println!("Updated {} (Registered '{}')", mod_rs_path.display(), component_name);
        }
    }
}

fn uppercase_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
