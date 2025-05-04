use super::{AsyncDispatcher, Transport};
use anyhow::Result;
use pb::{
    c2::{ReportTaskOutputRequest, TaskOutput},
    config::Config,
};
use prost_types::Timestamp;

/*
 * ReportFinishMessage indicates the end of a tome's evaluation.
 */
#[cfg_attr(debug_assertions, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct ReportFinishMessage {
    pub(crate) id: i64,
    pub(crate) exec_finished_at: Timestamp,
}

impl AsyncDispatcher for ReportFinishMessage {
    async fn dispatch(self, transport: &mut impl Transport, _cfg: Config) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: self.id,
                    output: String::new(),
                    exec_started_at: None,
                    exec_finished_at: Some(self.exec_finished_at),
                    error: None,
                }),
            })
            .await?;
        Ok(())
    }
}
