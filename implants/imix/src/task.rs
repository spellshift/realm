use anyhow::Result;
use c2::{
    pb::{ReportProcessListRequest, ReportTaskOutputRequest, TaskError, TaskOutput},
    TavernClient,
};
use eldritch::Output;
use tokio::task::JoinHandle;

/*
 * Task handle is responsible for tracking a running task and reporting it's output.
 */
pub struct TaskHandle {
    id: i64,
    handle: JoinHandle<()>,
    output: Output,
}

impl TaskHandle {
    // Track a new task handle.
    pub fn new(id: i64, output: Output, handle: JoinHandle<()>) -> TaskHandle {
        TaskHandle { id, handle, output }
    }

    // Returns true if the task has been completed, false otherwise.
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    // Report any available task output.
    pub async fn report(&mut self, tavern: &mut TavernClient) -> Result<()> {
        let exec_started_at = self.output.get_exec_started_at();
        let exec_finished_at = self.output.get_exec_finished_at();
        let text = self.output.collect();
        let err = self.output.collect_errors().pop().map(|err| TaskError {
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
        let process_lists = self.output.collect_process_lists();
        for list in process_lists {
            #[cfg(debug_assertions)]
            log::info!("reporting process list: len={}", list.list.len());

            tavern
                .report_process_list(ReportProcessListRequest {
                    task_id: self.id,
                    list: Some(list),
                })
                .await?;
        }

        Ok(())
    }
}
