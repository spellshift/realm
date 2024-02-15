use super::{Dispatcher, Transport};
use anyhow::Result;
use pb::c2::{ReportTaskOutputRequest, TaskOutput};
use prost_types::Timestamp;

/*
 * ReportFinishMessage indicates the end of a tome's evaluation.
 */
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct ReportFinishMessage {
    pub(crate) id: i64,
    pub(crate) exec_finished_at: Option<Timestamp>,
}

impl Dispatcher for ReportFinishMessage {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: self.id,
                    output: String::new(),
                    exec_started_at: None,
                    exec_finished_at: self.exec_finished_at,
                    error: None,
                }),
            })
            .await?;
        Ok(())
    }
}
