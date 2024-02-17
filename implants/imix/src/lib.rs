mod agent;
mod config;
mod install;
mod task;
mod version;
#[cfg(feature = "win_service")]
pub mod win_service;

use std::time::Duration;

pub use agent::Agent;
use clap::Command;
pub use config::Config;
pub use install::install;


pub async fn handle_main(){
    #[cfg(debug_assertions)]
    init_logging();

    if let Some(("install", _)) = Command::new("imix")
        .subcommand(Command::new("install").about("Install imix"))
        .get_matches()
        .subcommand()
    {
        install().await;
        return;
    }

    loop {
        let cfg = Config::default();
        let retry_interval = cfg.retry_interval;
        #[cfg(debug_assertions)]
        log::info!("agent config initialized {:#?}", cfg.clone());

        match run(cfg).await {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("callback loop fatal: {_err}");

                tokio::time::sleep(Duration::from_secs(retry_interval)).await;
            }
        }
    }
}

async fn run(cfg: Config) -> anyhow::Result<()> {
    let mut agent = Agent::new(cfg)?;
    agent.callback_loop().await?;
    Ok(())
}

#[cfg(debug_assertions)]
fn init_logging() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("IMIX_LOG")
        .init();
}
