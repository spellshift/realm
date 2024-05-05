use crate::runtime::{messages::ReverseShellPTYMessage, Environment};
use anyhow::Result;

pub fn reverse_shell_pty(env: &Environment, cmd: Option<String>) -> Result<()> {
    env.send(ReverseShellPTYMessage { id: env.id(), cmd })?;
    Ok(())
}
