use super::{SyncDispatcher, Transport};
use anyhow::Result;
use pb::config::Config;

/*
 * SetCallbackUriMessage sets the callback URI in the dispatched config.
 */
#[allow(dead_code)]
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct GetTasksMessage {
    pub(crate) id: i64,
}

impl SyncDispatcher for GetTasksMessage {
    fn dispatch(self, _transport: &mut impl Transport, cfg: Config) -> Result<Config> {
        Ok(cfg)
    }
}
