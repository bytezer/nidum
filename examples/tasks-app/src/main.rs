use nidum::prelude::*;
use std::sync::Arc;

#[derive(Injectable)]
pub struct ReportService;

#[tasks]
impl ReportService {
    // This will run every 5 seconds (Cron: Seconds Minutes Hours Day Month Year)
    #[cron("0/5 * * * * * *")]
    pub async fn generate_report(&self) {
        let now = nidum::__private::chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        println!("[{}] Generating background report...", now);
    }

    // Runs at the 30th second of every minute
    #[cron("30 * * * * * *")]
    pub async fn cleanup_temp_files(&self) {
        println!("🧹 Cleaning up temporary files...");
    }
}

#[module(
    providers = [SchedulerRegistry, ReportService],
    tasks = [ReportService]
)]
pub struct AppModule;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = NidumFactory::create()
        .register_module::<AppModule>(Arc::new(AppModuleContainer::new()));

    println!("Starting server on 127.0.0.1:3000");
    println!("Background tasks are now running. Waiting for trigger...");
    app.listen("127.0.0.1:3000").await?;

    Ok(())
}
