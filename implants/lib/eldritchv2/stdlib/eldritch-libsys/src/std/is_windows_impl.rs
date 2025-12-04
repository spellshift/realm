use anyhow::Result;

pub fn is_windows() -> Result<bool> {
    Ok(cfg!(target_os = "windows"))
}
