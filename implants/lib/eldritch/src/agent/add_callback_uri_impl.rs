use crate::runtime::{
    messages::{AddCallbackUriMessage, SyncMessage},
    Environment,
};
use anyhow::Result;

pub fn add_callback_uri(env: &Environment, new_uri: String) -> Result<()> {
    {
        let mut cfg = env
            .config
            .write()
            .map_err(|_| anyhow::anyhow!("failed to write config lock"))?;
        if !cfg.callback_uris.contains(&new_uri) {
            cfg.callback_uris.push(new_uri.clone());
        }
    }
    env.send(SyncMessage::from(AddCallbackUriMessage {
        id: env.id(),
        new_uri,
    }))?;
    Ok(())
}
