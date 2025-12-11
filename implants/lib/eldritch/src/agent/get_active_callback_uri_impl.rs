use crate::runtime::Environment;
use anyhow::Result;

pub fn get_active_callback_uri(env: &Environment) -> Result<String> {
    let cfg = env
        .config
        .read()
        .map_err(|_| anyhow::anyhow!("failed to read config lock"))?;
    cfg.active_uri()
        .ok_or_else(|| anyhow::anyhow!("no active callback uri"))
}
