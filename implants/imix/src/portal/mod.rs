use crate::shell::manager::ShellManagerMessage;
use anyhow::Result;
use eldritch_agent::Context;
use tokio::sync::mpsc;
use transport::Transport;

pub mod bytes;
pub mod pty;
pub mod run;
pub mod tcp;
pub mod udp;

pub async fn run_create_portal(
    context: Context,
    transport: Box<dyn Transport + Send + Sync>,
    shell_manager_tx: mpsc::Sender<ShellManagerMessage>,
) -> Result<()> {
    run::run(context, transport, shell_manager_tx).await
}
