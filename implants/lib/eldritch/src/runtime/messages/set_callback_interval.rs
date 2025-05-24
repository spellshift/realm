use super::{SyncDispatcher, Transport};
use anyhow::Result;
use pb::{c2::Beacon, config::Config};

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
        let b = match cfg.info {
            Some(i) => Ok(i),
            None => Err(anyhow::anyhow!(
                "SetCallbackIntervalMessage: beacon is missing from config"
            )),
        }?;
        c.info = Some(Beacon {
            identifier: b.identifier,
            principal: b.principal,
            host: b.host,
            agent: b.agent,
            interval: self.new_interval,
        });
        Ok(c)
    }
}
