pub mod bytes;
pub mod repl;
pub mod run;
pub mod shell;
pub mod tcp;
pub mod udp;

use anyhow::Result;
use transport::Transport;
// Added missing argument to `run_create_portal`
use crate::agent::ImixAgent;

pub async fn run_create_portal<T: Transport + Send + Sync + 'static>(
    task_id: i64,
    transport: T,
    agent: ImixAgent<T>,
) -> Result<()> {
    run::run(task_id, transport, agent).await
}
