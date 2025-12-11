use crate::runtime::{
    messages::{SetCallbackUriMessage, SyncMessage},
    Environment,
};
use anyhow::Result;

pub fn set_active_callback_uri(env: &Environment, new_uri: String) -> Result<()> {
    {
        let mut c = env
            .config
            .write()
            .map_err(|_| anyhow::anyhow!("failed to write config lock"))?;
        if let Some(idx) = c.callback_uris.iter().position(|u| u == &new_uri) {
            c.active_callback_uri_idx = idx as u64;
        } else {
            c.callback_uris.push(new_uri.clone());
            c.active_callback_uri_idx = (c.callback_uris.len() - 1) as u64;
        }
    }
    env.send(SyncMessage::from(SetCallbackUriMessage {
        id: env.id(),
        new_uri,
    }))?;
    Ok(())
}
