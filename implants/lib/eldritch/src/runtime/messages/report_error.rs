use super::{AsyncDispatcher, Transport};
use anyhow::Result;
use pb::{
    c2::{ReportTaskOutputRequest, TaskError, TaskOutput},
    config::Config,
};

/*
 * ReportErrorMessage reports an error encountered by this tome's evaluation.
 */
#[cfg_attr(any(debug_assertions, test), derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportErrorMessage {
    pub id: i64,
    pub error: String,
}

impl AsyncDispatcher for ReportErrorMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
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
