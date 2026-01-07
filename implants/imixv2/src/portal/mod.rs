use anyhow::Result;
use transport::Transport;

pub mod bytes;
pub mod run;
pub mod tcp;
pub mod udp;

pub async fn run_create_portal<T: Transport + Send + Sync + 'static>(
    task_id: i64,
    jwt: String,
    transport: T,
) -> Result<()> {
    run::run(task_id, transport).await
}
