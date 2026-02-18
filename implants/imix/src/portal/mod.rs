use anyhow::Result;
use pb::c2::TaskContext;
use transport::Transport;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use crate::shell::manager::ShellManagerMessage;

pub mod bytes;
pub mod run;
pub mod tcp;
pub mod udp;

pub async fn run_create_portal<T: Transport + Send + Sync + 'static>(
    task_context: TaskContext,
    transport: T,
    shell_manager_tx: Arc<Mutex<Option<mpsc::Sender<ShellManagerMessage>>>>,
) -> Result<()> {
    run::run(task_context, transport, shell_manager_tx).await
}
