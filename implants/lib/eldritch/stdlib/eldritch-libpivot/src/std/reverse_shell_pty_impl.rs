use alloc::string::String;
use alloc::sync::Arc;
use anyhow::Result;
use eldritch_agent::{Agent, Context};

pub fn reverse_shell_pty(
    agent: Arc<dyn Agent>,
    context: Context,
    cmd: Option<String>,
) -> Result<()> {
    agent
        .start_reverse_shell(context, cmd)
        .map_err(|e| anyhow::anyhow!(e))
}
