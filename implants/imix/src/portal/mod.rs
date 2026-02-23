use crate::shell::manager::ShellManagerMessage;
use anyhow::Result;
use eldritch_agent::Context;
use tokio::sync::mpsc;
use transport::Transport;

pub mod bytes;
pub mod run;
pub mod tcp;
pub mod udp;

pub async fn run_create_portal<T: Transport + Send + Sync + 'static>(
    context: Context,
    transport: T,
    shell_manager_tx: mpsc::Sender<ShellManagerMessage>,
) -> Result<()> {
    run::run(context, transport, shell_manager_tx).await
}
