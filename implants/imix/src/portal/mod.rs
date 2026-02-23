use crate::shell::manager::ShellManagerMessage;
use anyhow::Result;
use pb::c2::TaskContext;
use tokio::sync::mpsc;
use transport::Transport;

pub mod bytes;
pub mod run;
pub mod tcp;
pub mod udp;

pub async fn run_create_portal<T: Transport + Send + Sync + 'static>(
    task_context: TaskContext,
    transport: T,
    shell_manager_tx: mpsc::Sender<ShellManagerMessage>,
) -> Result<()> {
    run::run(task_context, transport, shell_manager_tx).await
}
