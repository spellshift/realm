use anyhow::Result;

pub fn hostname() -> Result<String> {
    Ok(whoami::fallible::hostname()?)
}
