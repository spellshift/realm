use super::{SyncDispatcher, Transport};
use anyhow::{Context, Result};
use pb::config::Config;

/*
 * SetCallbackIntervalMessage sets the callback interval in the dispatched config.
 */
#[allow(dead_code)]
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct SetCallbackIntervalMessage {
    pub(crate) id: i64,
    pub(crate) new_interval: u64,
}

impl SyncDispatcher for SetCallbackIntervalMessage {
    fn dispatch(self, _transport: &mut impl Transport, cfg: Config) -> Result<Config> {
        let mut c = cfg.clone();

        let info = c.info.as_mut().context("missing config info")?;
        let available_transports = info
            .available_transports
            .as_mut()
            .context("missing available transports")?;

        let active_index = available_transports.active_index as usize;
        let active_transport = available_transports
            .transports
            .get_mut(active_index)
            .context("active transport index out of bounds")?;

        active_transport.interval = self.new_interval;

        Ok(c)
    }
}
