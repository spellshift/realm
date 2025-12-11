use crate::runtime::Environment;
use anyhow::Result;

pub fn list_callback_uris(env: &Environment) -> Result<Vec<String>> {
    let cfg = env
        .config
        .read()
        .map_err(|_| anyhow::anyhow!("failed to read config lock"))?;
    Ok(cfg.callback_uris.clone())
}
