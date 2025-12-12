use anyhow::Result;

pub fn is_linux() -> Result<bool> {
    Ok(cfg!(target_os = "linux"))
}
