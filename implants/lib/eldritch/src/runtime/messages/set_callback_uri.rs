use super::{SyncDispatcher, Transport};
use anyhow::Result;
use pb::config::Config;

/*
 * SetCallbackUriMessage sets the callback URI in the dispatched config.
 */
#[allow(dead_code)]
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct SetCallbackUriMessage {
    pub(crate) id: i64,
    pub(crate) new_uri: String,
}

impl SyncDispatcher for SetCallbackUriMessage {
    fn dispatch(self, _transport: &mut impl Transport, cfg: Config) -> Result<Config> {
        let mut c = cfg.clone();
        c.callback_uri = self.new_uri;
        Ok(c)
    }
}
