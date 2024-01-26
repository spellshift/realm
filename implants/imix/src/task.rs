use anyhow::Result;
use c2::{
    pb::{ReportProcessListRequest, ReportTaskOutputRequest, TaskError, TaskOutput},
    TavernClient,
};
use eldritch::Output;
use tokio::task::JoinHandle;

pub struct TaskHandle {
    id: i64,
    handle: JoinHandle<()>,
    output: Output,
}

impl TaskHandle {
    pub fn new(id: i64, output: Output, handle: JoinHandle<()>) -> TaskHandle {
        TaskHandle { id, handle, output }
    }

    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub async fn report(&mut self, tavern: &mut TavernClient) -> Result<()> {
        // Report Task Output
        let exec_started_at = self.output.get_exec_started_at();
        let exec_finished_at = self.output.get_exec_finished_at();
        let err = match self.output.collect_errors().pop() {
            Some(err) => Some(TaskError {
                msg: err.to_string(),
            }),
            None => None,
        };
        let text = self.output.collect();
        if text.len() > 0
            || err.is_some()
            || exec_started_at.is_some()
            || exec_finished_at.is_some()
        {
            tavern
                .report_task_output(ReportTaskOutputRequest {
                    output: Some(TaskOutput {
                        id: self.id,
                        output: text.join(""),
                        error: err,
                        exec_started_at: exec_started_at,
                        exec_finished_at: exec_finished_at,
                    }),
                })
                .await?;
        }

        // Report Process Lists
        let process_lists = self.output.collect_process_lists();
        for list in process_lists {
            tavern
                .report_process_list(ReportProcessListRequest {
                    task_id: self.id,
                    list: Some(list),
                })
                .await?;
        }

        // Report Files TODO
        // let files = self.output.collect_files();
        // for f in files {
        //     tavern
        //         .report_file(ReportFileRequest {
        //             task_id: self.id,
        //             chunk: Some(f),
        //         })
        //         .await?;
        // }

        Ok(())
    }
}
