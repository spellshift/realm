use super::{Dispatcher, Transport};
use anyhow::Result;
use pb::c2::{ReportTaskOutputRequest, TaskOutput};
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

impl Dispatcher for ReportStartMessage {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
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
