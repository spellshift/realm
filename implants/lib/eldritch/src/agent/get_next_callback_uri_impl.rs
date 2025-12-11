use crate::runtime::Environment;
use anyhow::Result;

pub fn get_next_callback_uri(env: &Environment) -> Result<String> {
    let cfg = env
        .config
        .read()
        .map_err(|_| anyhow::anyhow!("failed to read config lock"))?;
    let len = cfg.callback_uris.len();
    if len == 0 {
        return Err(anyhow::anyhow!("no callback uris"));
    }
    let next_idx = (cfg.active_callback_uri_idx as usize + 1) % len;
    Ok(cfg.callback_uris[next_idx].clone())
}
