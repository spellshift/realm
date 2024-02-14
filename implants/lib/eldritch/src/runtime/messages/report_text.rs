use super::{Dispatcher, Transport};
use anyhow::Result;
use api::pb::c2::{ReportTaskOutputRequest, TaskOutput};
use prost_types::Timestamp;

pub struct ReportText {
    pub(crate) id: i64,
    pub(crate) text: String,
    pub(crate) exec_started_at: Option<Timestamp>,
    pub(crate) exec_finished_at: Option<Timestamp>,
}

impl Dispatcher for ReportText {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: self.id,
                    output: self.text,
                    exec_started_at: self.exec_started_at,
                    exec_finished_at: self.exec_finished_at,
                    error: None,
                }),
            })
            .await?;
        Ok(())
    }
}
