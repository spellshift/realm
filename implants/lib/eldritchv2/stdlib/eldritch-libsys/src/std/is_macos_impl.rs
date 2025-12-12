use anyhow::Result;

pub fn is_macos() -> Result<bool> {
    Ok(cfg!(target_os = "macos"))
}
