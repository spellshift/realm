use alloc::string::String;
use alloc::sync::Arc;
use anyhow::Result;
use eldritch_agent::Agent;

pub fn reverse_shell_pty(
    agent: Arc<dyn Agent>,
    task_id: i64,
    jwt: String,
    cmd: Option<String>,
) -> Result<()> {
    agent
        .start_reverse_shell(task_id, jwt, cmd)
        .map_err(|e| anyhow::anyhow!(e))
}
