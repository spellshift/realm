use crate::pb::{File, ProcessList};
use anyhow::Result;
use std::{sync::mpsc, time::Duration};

pub struct Receiver {
    outputs: mpsc::Receiver<String>,
    process_lists: mpsc::Receiver<ProcessList>,
    files: mpsc::Receiver<File>,
}

impl Receiver {
    /*
     * Collects all currently available reported output
     */
    pub fn collect_output(&self) -> Result<Vec<String>> {
        drain(&self.outputs)
    }

    /*
     * Returns all currently available reported process lists, if any.
     */
    pub fn collect_process_lists(&self) -> Result<Vec<ProcessList>> {
        drain(&self.process_lists)
    }

    /*
     * Returns all currently available reported files, if any.
     */
    pub fn collect_files(&self) -> Result<Vec<File>> {
        drain(&self.files)
    }
}

pub struct Sender {
    outputs: mpsc::Sender<String>,
    process_lists: mpsc::Sender<ProcessList>,
    files: mpsc::Sender<File>,
}

impl Sender {
    /*
     * Report output of the tome execution.
     */
    pub fn report_output(&self, output: String) -> Result<()> {
        self.outputs.send(output)?;
        Ok(())
    }
    /*
     * Report a process list that was collected by the tome.
     */
    pub fn report_process_list(&self, processes: ProcessList) -> Result<()> {
        self.process_lists.send(processes)?;
        Ok(())
    }

    /*
     * Report a file that was collected by the tome.
     */
    pub fn report_file(&self, f: File) -> Result<()> {
        self.files.send(f)?;
        Ok(())
    }
}

pub fn channel() -> (Sender, Receiver) {
    let (output_sender, output_receiver) = mpsc::channel::<String>();
    let (process_list_sender, process_list_receiver) = mpsc::channel::<ProcessList>();
    let (file_sender, file_receiver) = mpsc::channel::<File>();

    return (
        Sender {
            outputs: output_sender,
            process_lists: process_list_sender,
            files: file_sender,
        },
        Receiver {
            outputs: output_receiver,
            process_lists: process_list_receiver,
            files: file_receiver,
        },
    );
}

/*
 * Drain a receiver, returning all currently available results as a Vec.
 */
fn drain<T>(reciever: &mpsc::Receiver<T>) -> Result<Vec<T>> {
    let mut result: Vec<T> = Vec::new();
    loop {
        let val = match reciever.recv_timeout(Duration::from_millis(100)) {
            Ok(v) => v,
            Err(err) => {
                match err.to_string().as_str() {
                    "channel is empty and sending half is closed" => {
                        break;
                    }
                    "timed out waiting on channel" => {
                        break;
                    }
                    _ => {
                        #[cfg(debug_assertions)]
                        eprint!("Error draining channel: {}", err)
                    }
                }
                break;
            }
        };
        // let appended_line = format!("{}{}", res.to_owned(), new_res_line);
        result.push(val);
    }
    Ok(result)
}
