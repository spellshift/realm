use super::{SyncDispatcher, Transport};
use anyhow::Result;
use pb::config::Config;

/*
 * AddCallbackUriMessage appends a new callback URI to the dispatched config.
 */
#[allow(dead_code)]
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct AddCallbackUriMessage {
    pub(crate) id: i64,
    pub(crate) new_uri: String,
}

impl SyncDispatcher for AddCallbackUriMessage {
    fn dispatch(self, _transport: &mut impl Transport, cfg: Config) -> Result<Config> {
        let mut c = cfg.clone();
        if !c.callback_uris.contains(&self.new_uri) {
            c.callback_uris.push(self.new_uri);
        }
        Ok(c)
    }
}
