use crate::pb::{File, ProcessList};
use anyhow::{Context, Error, Result};
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
    pub(super) tx_output: Sender<String>,
    pub(super) tx_error: Sender<Error>,
    pub(super) tx_process_list: Sender<ProcessList>,
    pub(super) tx_file: Sender<File>,
    pub(super) tx_file_request: Sender<FileRequest>,
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

    /*
     * Report output of the tome execution.
     */
    pub fn report_output(&self, output: String) -> Result<()> {
        self.tx_output.send(output)?;
        Ok(())
    }

    /*
     * Report error during tome execution.
     */
    pub fn report_error(&self, err: anyhow::Error) -> Result<()> {
        self.tx_error.send(err)?;
        Ok(())
    }

    /*
     * Report a process list that was collected by the tome.
     */
    pub fn report_process_list(&self, processes: ProcessList) -> Result<()> {
        self.tx_process_list.send(processes)?;
        Ok(())
    }

    /*
     * Report a file that was collected by the tome.
     */
    pub fn report_file(&self, f: File) -> Result<()> {
        self.tx_file.send(f)?;
        Ok(())
    }

    /*
     * Request a file from the caller of this runtime.
     * This will return a channel of file chunks.
     */
    pub fn request_file(&self, name: String) -> Result<Receiver<Vec<u8>>> {
        let (tx_data, data) = channel::<Vec<u8>>();
        self.tx_file_request.send(FileRequest { name, tx_data })?;
        Ok(data)
    }
}

/*
 * Enables Environment to be used as a starlark print handler.
 */
impl PrintHandler for Environment {
    fn println(&self, text: &str) -> Result<()> {
        self.report_output(text.to_string())?;

        #[cfg(feature = "print_stdout")]
        print!("{}", text);

        Ok(())
    }
}
