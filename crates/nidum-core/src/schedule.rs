use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use cron::Schedule;
use std::str::FromStr;
use tokio::time::sleep;

use crate::Injectable;

pub trait TaskRegistrar<C>: Send + Sync + 'static {
    fn register_tasks(self: Arc<Self>, container: &C);
}

pub type CronCallback = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Clone)]
pub struct SchedulerRegistry {
    // In the future, this can hold a HashMap of job handles to allow pausing/canceling.
}

impl SchedulerRegistry {
    pub fn new() -> Self {
        Self {}
    }

    pub fn register_cron(&self, name: &'static str, cron_expr: &str, callback: CronCallback) {
        let schedule = match Schedule::from_str(cron_expr) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to parse cron expression for task '{}': {}", name, e);
                return;
            }
        };

        let callback = Arc::new(callback);

        tokio::spawn(async move {
            let mut upcoming = schedule.upcoming(chrono::Utc);
            loop {
                if let Some(next_dt) = upcoming.next() {
                    let now = chrono::Utc::now();
                    if next_dt > now {
                        if let Ok(duration) = (next_dt - now).to_std() {
                            sleep(duration).await;
                            let cb = Arc::clone(&callback);
                            // Run the task
                            cb().await;
                        }
                    }
                } else {
                    break;
                }
            }
        });
    }
}

impl<C> Injectable<C> for SchedulerRegistry {
    fn create(_: &C) -> Arc<Self> {
        Arc::new(Self::new())
    }
}
