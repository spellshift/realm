use super::messages::{AsyncMessage, Message, ReportTextMessage};
use anyhow::{Context, Result};
use pb::config::Config;
use starlark::{
    values::{AnyLifetime, ProvidesStaticType},
    PrintHandler,
};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

#[allow(clippy::needless_lifetimes)]
#[derive(ProvidesStaticType)]
pub struct Environment {
    pub(super) id: i64,
    pub(super) tx: Sender<Message>,
    pub(super) config: Arc<RwLock<Config>>,
}

impl Environment {
    /*
     * Extract an existing runtime environment from the starlark evaluator extra field.
     */
    pub fn from_extra<'a>(extra: Option<&'a dyn AnyLifetime<'a>>) -> Result<&'a Environment> {
        extra
            .context("no extra field present in evaluator")?
            .downcast_ref::<Environment>()
            .context("no runtime client present in evaluator")
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn send(&self, msg: impl Into<Message>) -> Result<()> {
        self.tx.send(msg.into())?;
        Ok(())
    }

    #[cfg(test)]
    pub fn mock(id: i64, tx: Sender<Message>) -> Self {
        use pb::config::CALLBACK_URI;
        let cfg = Config {
            info: None,
            callback_uris: vec![String::from(CALLBACK_URI)],
            proxy_uri: None,
            retry_interval: 5,
            run_once: false,
            active_callback_uri_idx: 0,
            transport_schemes: Vec::new(),
        };
        Self {
            id,
            tx,
            config: Arc::new(RwLock::new(cfg)),
        }
    }
}

/*
 * Enables Environment to be used as a starlark print handler.
 */
impl PrintHandler for Environment {
    fn println(&self, text: &str) -> Result<()> {
        self.send(AsyncMessage::from(ReportTextMessage {
            id: self.id,
            text: format!("{}\n", text),
        }))?;

        #[cfg(feature = "print_stdout")]
        println!("{}", text);

        Ok(())
    }
}
