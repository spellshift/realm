use anyhow::Result;

pub fn is_linux() -> Result<bool> {
    if cfg!(target_os = "linux") {
        return Ok(true);
    }
    Ok(false)
}