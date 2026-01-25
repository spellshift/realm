use anyhow::Result;
use pb::c2::TaskContext;
use transport::Transport;

pub mod bytes;
pub mod run;
pub mod tcp;
pub mod udp;

pub async fn run_create_portal<T: Transport + Send + Sync + 'static>(
    task_context: TaskContext,
    transport: T,
) -> Result<()> {
    run::run(task_context, transport).await
}
