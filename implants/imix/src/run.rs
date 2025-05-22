use clap::Command;
use std::time::Duration;

pub use crate::agent::Agent;
pub use crate::install::install;
use crate::version::VERSION;
pub use pb::config::Config;

use transport::{Transport, GRPC};

pub async fn handle_main() {
    if let Some(("install", _)) = Command::new("imix")
        .subcommand(Command::new("install").about("Install imix"))
        .get_matches()
        .subcommand()
    {
        install().await;
        return;
    }

    loop {
        let cfg = Config::default_with_imix_verison(VERSION);
        let retry_interval = cfg.retry_interval;
        #[cfg(debug_assertions)]
        log::info!("agent config initialized {:#?}", cfg.clone());

        let run_once = cfg.run_once;

        match run(cfg).await {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                log::error!("callback loop fatal: {_err}");

                tokio::time::sleep(Duration::from_secs(retry_interval)).await;
            }
        }

        if run_once {
            break;
        }
    }
}

async fn run(cfg: Config) -> anyhow::Result<()> {
    let mut agent = Agent::new(cfg, GRPC::init())?;
    agent.callback_loop().await?;
    Ok(())
}

#[cfg(debug_assertions)]
pub fn init_logging() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("IMIX_LOG")
        .init();
}
