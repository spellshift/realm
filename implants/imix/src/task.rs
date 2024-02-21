use anyhow::Result;
use eldritch::runtime::messages::Dispatcher;
use pb::c2::{ReportTaskOutputRequest, TaskError, TaskOutput};
use transport::Transport;

/*
 * Task handle is responsible for tracking a running task and reporting it's output.
 */
pub struct TaskHandle {
    id: i64,
    runtime: eldritch::Runtime,
    pool: tokio::task::JoinSet<()>,
}

impl TaskHandle {
    // Track a new task handle.
    pub fn new(id: i64, runtime: eldritch::Runtime) -> TaskHandle {
        TaskHandle {
            id,
            runtime,
            pool: tokio::task::JoinSet::new(),
        }
    }

    // Returns true if the task has been completed, false otherwise.
    pub fn is_finished(&self) -> bool {
        // Check Report Pool
        if !self.pool.is_empty() {
            return false;
        }

        // Check Tome Evaluation
        self.runtime.is_finished()
    }

    // Report any available task output.
    // Also responsible for downloading any files requested by the eldritch runtime.
    pub async fn report(&mut self, tavern: &mut (impl Transport + 'static)) -> Result<()> {
        let messages = self.runtime.collect();
        for msg in messages {
            // Copy values for logging
            let id = self.id;
            let msg_str = msg.to_string();

            // Each message is dispatched in it's own tokio task, managed by this task handle's pool.
            let mut t = tavern.clone();
            self.pool.spawn(async move {
                match msg.dispatch(&mut t).await {
                    Ok(_) => {
                        #[cfg(debug_assertions)]
                        log::info!("message success (task_id={},msg={})", id, msg_str);
                    }
                    Err(err) => {
                        #[cfg(debug_assertions)]
                        log::error!(
                            "message failed (task_id={},msg={}): {}",
                            id,
                            msg_str.clone(),
                            err
                        );

                        // Attempt to report this dispatch error to the server
                        // This will help in cases where one transport method is failing but we can
                        // still report errors.
                        match t
                            .report_task_output(ReportTaskOutputRequest {
                                output: Some(TaskOutput {
                                    id: id,
                                    output: String::new(),
                                    error: Some(TaskError {
                                        msg: format!("dispatch error ({}): {:#?}", msg_str, err),
                                    }),
                                    exec_started_at: None,
                                    exec_finished_at: None,
                                }),
                            })
                            .await
                        {
                            Ok(_) => {}
                            Err(_err) => {
                                #[cfg(debug_assertions)]
                                log::error!("failed to report dispatch error: {}", _err);
                            }
                        };
                    }
                }
            });
        }
        Ok(())
    }
}
