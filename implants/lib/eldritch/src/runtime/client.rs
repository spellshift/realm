use crate::pb::{File, ProcessList};
use anyhow::{Context, Error, Result};
use starlark::values::{AnyLifetime, ProvidesStaticType};
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct FileRequest {
    name: String,
    ch_data: Sender<Vec<u8>>,
}

impl FileRequest {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn send_chunk(&self, chunk: Vec<u8>) -> Result<()> {
        self.ch_data.send(chunk)?;
        Ok(())
    }
}

#[derive(ProvidesStaticType)]
pub struct Client {
    pub(super) ch_output: Sender<String>,
    pub(super) ch_error: Sender<Error>,
    pub(super) ch_process_list: Sender<ProcessList>,
    pub(super) ch_file: Sender<File>,
    pub(super) ch_file_requests: Sender<FileRequest>,
}

impl Client {
    /*
     * Extract an existing runtime client from the starlark evaluator extra field.
     */
    pub fn from_extra<'a>(extra: Option<&'a dyn AnyLifetime<'a>>) -> Result<&'a Client> {
        extra
            .context("no extra field present in evaluator")?
            .downcast_ref::<Client>()
            .context("no runtime client present in evaluator")
    }

    /*
     * Report output of the tome execution.
     */
    pub fn report_output(&self, output: String) -> Result<()> {
        self.ch_output.send(output)?;
        Ok(())
    }

    /*
     * Report error of the tome execution.
     */
    pub fn report_error(&self, err: anyhow::Error) -> Result<()> {
        self.ch_error.send(err)?;
        Ok(())
    }

    /*
     * Report a process list that was collected by the tome.
     */
    pub fn report_process_list(&self, processes: ProcessList) -> Result<()> {
        self.ch_process_list.send(processes)?;
        Ok(())
    }

    /*
     * Report a file that was collected by the tome.
     */
    pub fn report_file(&self, f: File) -> Result<()> {
        self.ch_file.send(f)?;
        Ok(())
    }

    /*
     * Request a file from the caller of this runtime.
     * This will return a channel of file chunks.
     */
    pub fn request_file(&self, name: String) -> Result<Receiver<Vec<u8>>> {
        let (ch_data, data) = channel::<Vec<u8>>();
        self.ch_file_requests.send(FileRequest { name, ch_data })?;
        Ok(data)
    }
}
