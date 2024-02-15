use super::messages::{Message, ReportTextMessage};
use anyhow::{Context, Result};

use starlark::{
    values::{AnyLifetime, ProvidesStaticType},
    PrintHandler,
};
use std::sync::mpsc::Sender;

pub struct FileRequest {
    name: String,
    tx_data: Sender<Vec<u8>>,
}

impl FileRequest {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn send_chunk(&self, chunk: Vec<u8>) -> Result<()> {
        self.tx_data.send(chunk)?;
        Ok(())
    }
}

#[derive(ProvidesStaticType)]
pub struct Environment {
    pub(super) id: i64,
    pub(super) tx: Sender<Message>,
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
}

/*
 * Enables Environment to be used as a starlark print handler.
 */
impl PrintHandler for Environment {
    fn println(&self, text: &str) -> Result<()> {
        self.send(ReportTextMessage {
            id: self.id,
            text: String::from(text),
        })?;

        #[cfg(feature = "print_stdout")]
        print!("{}", text);

        Ok(())
    }
}
