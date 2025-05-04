use super::{Dispatcher, Transport};
use anyhow::Result;
use pb::{c2::Beacon, config::Config};

/*
 * SetCallbackIntervalMessage sets the callback interval in the dispatched config.
 */
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct SetCallbackIntervalMessage {
    pub(crate) id: i64,
    pub(crate) new_interval: u64,
}

impl SetCallbackIntervalMessage {
    pub fn refresh_config(self, cfg: Config) -> Result<Config> {
        let mut c = cfg.clone();
        c.retry_interval = self.new_interval;
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

impl Dispatcher for SetCallbackIntervalMessage {
    // will likley never be called but nonethless is a noop
    async fn dispatch(self, _transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        Ok(())
    }
}
