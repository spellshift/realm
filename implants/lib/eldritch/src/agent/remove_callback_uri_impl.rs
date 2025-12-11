use crate::runtime::{
    messages::{RemoveCallbackUriMessage, SyncMessage},
    Environment,
};
use anyhow::Result;

pub fn remove_callback_uri(env: &Environment, uri: String) -> Result<()> {
    {
        let mut c = env
            .config
            .write()
            .map_err(|_| anyhow::anyhow!("failed to write config lock"))?;
        if let Some(idx) = c.callback_uris.iter().position(|u| u == &uri) {
            c.callback_uris.remove(idx);
            let len = c.callback_uris.len() as u64;
            let current = c.active_callback_uri_idx;

            if len == 0 {
                c.active_callback_uri_idx = 0;
            } else if current > idx as u64 {
                c.active_callback_uri_idx = current - 1;
            } else if current == idx as u64 {
                if current >= len {
                    c.active_callback_uri_idx = 0;
                }
            }
        }
    }
    env.send(SyncMessage::from(RemoveCallbackUriMessage {
        id: env.id(),
        uri,
    }))?;
    Ok(())
}
