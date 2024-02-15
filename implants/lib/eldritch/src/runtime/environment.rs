use super::messages::{Message, ReportText};
use anyhow::{Context, Error, Result};
use pb::eldritch::{Credential, File, ProcessList};
use starlark::{
    values::{AnyLifetime, ProvidesStaticType},
    PrintHandler,
};
use std::sync::mpsc::{channel, Receiver, Sender};

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

    pub fn send(&self, msg: Message) -> Result<()> {
        self.tx.send(msg)?;
        Ok(())
    }
}

/*
 * Enables Environment to be used as a starlark print handler.
 */
impl PrintHandler for Environment {
    fn println(&self, text: &str) -> Result<()> {
        self.send(Message::ReportText(ReportText {
            id: self.id,
            text: String::from(text),
            exec_started_at: None,
            exec_finished_at: None,
        }))?;

        #[cfg(feature = "print_stdout")]
        print!("{}", text);

        Ok(())
    }
}
