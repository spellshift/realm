use super::{SyncDispatcher, Transport};
use anyhow::Result;
use pb::config::Config;

/*
 * RemoveCallbackUriMessage removes a callback URI from the dispatched config.
 */
#[allow(dead_code)]
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct RemoveCallbackUriMessage {
    pub(crate) id: i64,
    pub(crate) uri: String,
}

impl SyncDispatcher for RemoveCallbackUriMessage {
    fn dispatch(self, _transport: &mut impl Transport, cfg: Config) -> Result<Config> {
        let mut c = cfg.clone();
        if let Some(idx) = c.callback_uris.iter().position(|u| u == &self.uri) {
            c.callback_uris.remove(idx);
            let len = c.callback_uris.len() as u64;
            let current = c.active_callback_uri_idx;

            if len == 0 {
                c.active_callback_uri_idx = 0;
            } else if current > idx as u64 {
                c.active_callback_uri_idx = current - 1;
            } else if current == idx as u64 {
                // Active was removed.
                if current >= len {
                    c.active_callback_uri_idx = 0;
                }
            }
        }
        Ok(c)
    }
}
