use anyhow::Result;

pub fn is_macos() -> Result<bool> {
    if cfg!(target_os = "macos") {
        return Ok(true);
    }
    Ok(false)
}