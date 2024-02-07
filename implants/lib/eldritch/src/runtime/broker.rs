use super::FileRequest;
use crate::pb::{File, ProcessList};
use anyhow::Error;
use prost_types::Timestamp;
use std::sync::mpsc::Receiver;
use std::time::Duration;

/*
 * A Broker to the runtime enables callers to interact with in-progress eldritch execution.
 * This enables the realtime collection of output and structured output,
 * as well as providing resources (such as files) to the runtime.
 * Each of the `collect` methods will return lists of all currently available data.
 */
pub struct Broker {
    pub(super) exec_started_at: Receiver<Timestamp>,
    pub(super) exec_finished_at: Receiver<Timestamp>,
    pub(super) outputs: Receiver<String>,
    pub(super) errors: Receiver<Error>,
    pub(super) process_lists: Receiver<ProcessList>,
    pub(super) files: Receiver<File>,
    pub(super) file_requests: Receiver<FileRequest>,
}

impl Broker {
    /*
     * Returns the timestamp of when execution started, if available.
     */
    pub fn get_exec_started_at(&self) -> Option<Timestamp> {
        drain_last(&self.exec_started_at)
    }

    /*
     * Returns the timestamp of when execution finished, if available.
     */
    pub fn get_exec_finished_at(&self) -> Option<Timestamp> {
        drain_last(&self.exec_finished_at)
    }

    /*
     * Collects all currently available reported text output.
     */
    pub fn collect_text(&self) -> Vec<String> {
        drain(&self.outputs)
    }

    /*
     * Collects all currently available reported errors, if any.
     */
    pub fn collect_errors(&self) -> Vec<Error> {
        drain(&self.errors)
    }

    /*
     * Returns all currently available reported process lists, if any.
     */
    pub fn collect_process_lists(&self) -> Vec<ProcessList> {
        drain(&self.process_lists)
    }

    /*
     * Returns all currently available reported files, if any.
     */
    pub fn collect_files(&self) -> Vec<File> {
        drain(&self.files)
    }

    /*
     * Returns all FileRequests that the eldritch runtime has requested, if any.
     */
    pub fn collect_file_requests(&self) -> Vec<FileRequest> {
        drain(&self.file_requests)
    }
}

/*
 * Drain a receiver, returning only the last currently available result.
 */
fn drain_last<T>(receiver: &Receiver<T>) -> Option<T> {
    drain(receiver).pop()
}

/*
 * Drain a receiver, returning all currently available results as a Vec.
 */
fn drain<T>(reciever: &Receiver<T>) -> Vec<T> {
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
                        eprint!("failed to drain channel: {}", err)
                    }
                }
                break;
            }
        };
        // let appended_line = format!("{}{}", res.to_owned(), new_res_line);
        result.push(val);
    }
    result
}
