use crate::runtime::{messages::ReverseShellMessage, Environment};
use anyhow::Result;

pub fn reverse_shell(env: &Environment) -> Result<()> {
    env.send(ReverseShellMessage { id: env.id() })?;
    Ok(())
}
