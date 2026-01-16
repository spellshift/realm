use super::{AsyncDispatcher, Transport};
use anyhow::Result;
use pb::{
    c2::{ReportTaskOutputRequest, TaskContext, TaskError, TaskOutput},
    config::Config,
};

/*
 * ReportErrorMessage reports an error encountered by this tome's evaluation.
 */
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportErrorMessage {
    pub id: i64,
    pub error: String,
}

impl AsyncDispatcher for ReportErrorMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
                context: Some(TaskContext {
                    task_id: self.id,
                    jwt: "no_jwt".to_string(),
                }),
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
