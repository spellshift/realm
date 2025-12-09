use super::{AsyncDispatcher, Transport};
use anyhow::Result;
use pb::{
    c2::{ReportTaskOutputRequest, TaskOutput},
    config::Config,
};
use prost_types::Timestamp;

/*
 * ReportStartMessage indicates the start of a tome's evaluation.
 */
#[cfg_attr(any(debug_assertions, test), derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportStartMessage {
    pub(crate) id: i64,
    pub(crate) exec_started_at: Timestamp,
}

impl AsyncDispatcher for ReportStartMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
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
