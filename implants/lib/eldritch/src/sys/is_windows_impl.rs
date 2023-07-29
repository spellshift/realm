use anyhow::Result;

pub fn is_windows() -> Result<bool> {
    if cfg!(target_os = "windows") {
        return Ok(true);
    }
    Ok(false)
}
