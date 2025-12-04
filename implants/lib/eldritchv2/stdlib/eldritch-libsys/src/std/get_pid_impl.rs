use anyhow::Result;

pub fn get_pid() -> Result<u32> {
    Ok(std::process::id())
}
