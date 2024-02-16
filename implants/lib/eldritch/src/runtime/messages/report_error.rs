use super::{Dispatcher, Transport};
use anyhow::Result;
use pb::c2::{ReportTaskOutputRequest, TaskError, TaskOutput};

/*
 * ReportErrorMessage reports an error encountered by this tome's evaluation.
 */
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportErrorMessage {
    pub(crate) id: i64,
    pub error: String,
}

impl Dispatcher for ReportErrorMessage {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: self.id,
                    output: String::from(""),
                    exec_started_at: None,
                    exec_finished_at: None,
                    error: Some(TaskError { msg: self.error }),
                }),
            })
            .await?;
        Ok(())
    }
}
