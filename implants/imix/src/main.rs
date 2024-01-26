use anyhow::Result;
use imix::{Agent, Config};

#[tokio::main(flavor = "multi_thread", worker_threads = 128)]
async fn main() {
    loop {
        match run().await {
            Ok(_) => {}
            Err(_err) => {
                #[cfg(debug_assertions)]
                eprint!("callback loop fatal error: {}", _err)
            }
        }
    }
}

async fn run() -> Result<()> {
    let cfg = Config::default();
    let mut agent = Agent::gen_from_config(cfg).await?;

    agent.callback_loop().await;
    Ok(())
}
