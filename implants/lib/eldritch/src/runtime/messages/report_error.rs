use super::{Dispatcher, Transport};
use anyhow::{Error, Result};
use api::pb::c2::{ReportTaskOutputRequest, TaskError, TaskOutput};
use prost_types::Timestamp;

pub struct ReportError {
    pub(crate) id: i64,
    pub(crate) error: Error,
    pub(crate) exec_started_at: Option<Timestamp>,
    pub(crate) exec_finished_at: Option<Timestamp>,
}

impl Dispatcher for ReportError {
    async fn dispatch(self, transport: &mut impl Transport) -> Result<()> {
        transport
            .report_task_output(ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: self.id,
                    output: String::from(""),
                    exec_started_at: self.exec_started_at,
                    exec_finished_at: self.exec_finished_at,
                    error: Some(TaskError {
                        msg: self.error.to_string(),
                    }),
                }),
            })
            .await?;
        Ok(())
    }
}
