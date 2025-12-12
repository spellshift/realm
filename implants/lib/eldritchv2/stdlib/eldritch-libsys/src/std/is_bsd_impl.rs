use anyhow::Result;

pub fn is_bsd() -> Result<bool> {
    if cfg!(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )) {
        return Ok(true);
    }
    Ok(false)
}
