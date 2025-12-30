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
    fn dispatch(self, transport: &mut impl Transport, cfg: Config) -> Result<Config> {
        let mut c = cfg.clone();
        let b = match cfg.info {
            Some(i) => Ok(i),
            None => Err(anyhow::anyhow!(
                "SetCallbackIntervalMessage: beacon is missing from config"
            )),
        }?;
        // TODO: we can probably just modify the interval not rebuild the entire beacon see set_callback_uri.rs
        c.info = Some(Beacon {
            identifier: b.identifier,
            principal: b.principal,
            host: b.host,
            agent: b.agent,
            active_transport: Some(pb::c2::ActiveTransport {
                uri: b
                    .active_transport
                    .as_ref()
                    .map_or(String::new(), |at| at.uri.clone()),
                interval: self.new_interval,
                r#type: transport.get_type() as i32,
                extra: b
                    .active_transport
                    .as_ref()
                    .map_or(String::new(), |at| at.extra.clone()),
            }),
        });
        Ok(c)
    }
}
