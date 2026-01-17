use super::{AsyncDispatcher, Transport};
use anyhow::Result;
use pb::{
    c2::{ReportTaskOutputRequest, TaskContext, TaskOutput},
    config::Config,
};
use prost_types::Timestamp;

/*
 * ReportStartMessage indicates the start of a tome's evaluation.
 */
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportStartMessage {
    pub(crate) id: i64,
    pub(crate) exec_started_at: Timestamp,
}

impl AsyncDispatcher for ReportStartMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
                context: Some(TaskContext {
                    task_id: self.id,
                    jwt: "no_jwt".to_string(),
                }),
                output: Some(TaskOutput {
                    id: self.id,
                    output: String::new(),
                    exec_started_at: Some(self.exec_started_at),
                    exec_finished_at: None,
                    error: None,
                }),
            })
            .await?;
        Ok(())
    }
}
